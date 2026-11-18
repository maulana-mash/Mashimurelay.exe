use base64::Engine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenFileResponse {
    pub file_name: String,
    pub file_path: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemAppInfo {
    pub name: String,
    pub version: String,
    pub os: String,
    pub arch: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmissionRecord {
    pub student_name: String,
    pub presensi_no: String,
    pub filename: String,
    pub file_path: String,
    pub timestamp: String,
    pub content: String,
    #[serde(default)]
    pub revision: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClassroomServerInfo {
    pub ip: String,
    pub port: u16,
    pub url: String,
    pub task_name: String,
    pub save_dir: String,
    #[serde(default)]
    pub exam_mode: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClassroomStatusResponse {
    pub running: bool,
    pub server_info: Option<ClassroomServerInfo>,
    pub submissions: Vec<SubmissionRecord>,
    pub exam_mode: bool,
}

pub struct AppClassroomState {
    pub running: Arc<AtomicBool>,
    pub server_info: Arc<Mutex<Option<ClassroomServerInfo>>>,
    pub task_json: Arc<Mutex<String>>,
    pub submissions: Arc<Mutex<Vec<SubmissionRecord>>>,
    pub save_dir: Arc<Mutex<String>>,
    pub active_port: Arc<Mutex<u16>>,
    pub exam_mode: Arc<AtomicBool>,
}

fn get_local_ip() -> String {
    let targets = [
        "192.168.137.1:80", // Windows 10/11 Mobile Hotspot
        "192.168.43.1:80",  // Android Hotspot
        "172.20.10.1:80",   // iOS Hotspot
        "192.168.1.1:80",   // Standard Router Subnet 1
        "192.168.0.1:80",   // Standard Router Subnet 0
        "192.168.11.1:80",  // School/Lab Wi-Fi Subnet 11
        "192.168.100.1:80", // Fiber/ONT Subnet 100
        "10.0.0.1:80",      // Enterprise LAN 10.x
        "8.8.8.8:80",       // Public DNS 1
        "1.1.1.1:80",       // Public DNS 2
    ];

    for target in targets {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            if socket.connect(target).is_ok() {
                if let Ok(addr) = socket.local_addr() {
                    let ip = addr.ip().to_string();
                    if !ip.starts_with("127.") && !ip.starts_with("169.254.") && ip != "0.0.0.0" {
                        return ip;
                    }
                }
            }
        }
    }
    "127.0.0.1".to_string()
}

fn get_default_save_dir(task_name: &str) -> std::path::PathBuf {
    let mut base = if let Ok(doc) = std::env::var("USERPROFILE") {
        std::path::PathBuf::from(doc).join("Documents")
    } else {
        std::path::PathBuf::from(".")
    };
    base.push("Mashimurelay");
    base.push("Tugas_Siswa");
    let safe_name: String = task_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    base.push(if safe_name.is_empty() { "Tugas_Praktikum" } else { &safe_name });
    base
}

fn handle_http_client(
    stream: &mut TcpStream,
    task_json_ref: Arc<Mutex<String>>,
    submissions_ref: Arc<Mutex<Vec<SubmissionRecord>>>,
    save_dir_ref: Arc<Mutex<String>>,
    exam_mode_ref: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0u8; 8192];
    let bytes_read = stream.read(&mut buffer)?;
    if bytes_read == 0 {
        return Ok(());
    }

    let req_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut lines = req_str.lines();
    let first_line = lines.next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Ok(());
    }

    let method = parts[0];
    let path = parts[1];
    let is_exam = exam_mode_ref.load(Ordering::SeqCst);

    if method == "OPTIONS" {
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\nConnection: close\r\n\r\n";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "GET" && (path == "/task.smrl" || path == "/") {
        let content = task_json_ref.lock().unwrap().clone();
        let content_bytes = content.as_bytes();
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Expose-Headers: X-Exam-Mode\r\nX-Exam-Mode: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            is_exam,
            content_bytes.len(),
            content
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "GET" && path == "/info" {
        let resp_json = format!(
            "{{\"status\":\"ok\",\"exam_mode\":{}}}",
            is_exam
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Expose-Headers: X-Exam-Mode\r\nX-Exam-Mode: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            is_exam,
            resp_json.len(),
            resp_json
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "POST" && path == "/submit" {
        let mut content_length = 0usize;
        for line in lines {
            let lower = line.to_lowercase();
            if lower.starts_with("content-length:") {
                if let Some(val) = line.split(':').nth(1) {
                    content_length = val.trim().parse().unwrap_or(0);
                }
            }
        }

        let header_end = req_str
            .find("\r\n\r\n")
            .map(|idx| idx + 4)
            .or_else(|| req_str.find("\n\n").map(|idx| idx + 2))
            .unwrap_or(bytes_read);

        let mut body_bytes = buffer[header_end..bytes_read].to_vec();
        while body_bytes.len() < content_length {
            let mut chunk = [0u8; 8192];
            let n = stream.read(&mut chunk)?;
            if n == 0 {
                break;
            }
            body_bytes.extend_from_slice(&chunk[..n]);
        }

        let body_str = String::from_utf8_lossy(&body_bytes);
        if let Ok(payload) = serde_json::from_str::<serde_json::Value>(&body_str) {
            let s_name = payload
                .get("student_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Siswa")
                .to_string();
            let p_no = payload
                .get("presensi_no")
                .and_then(|v| v.as_str())
                .unwrap_or("-")
                .to_string();
            let f_name = payload
                .get("filename")
                .and_then(|v| v.as_str())
                .unwrap_or("tugas.smrl")
                .to_string();
            let s_content = payload
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let safe_filename = if f_name.ends_with(".smrl") {
                f_name
            } else {
                format!("{}.smrl", f_name)
            };

            let mut subs = submissions_ref.lock().unwrap();
            let existing_pos = subs
                .iter()
                .position(|r| r.presensi_no == p_no && r.student_name == s_name);

            if is_exam && existing_pos.is_some() {
                let resp = "HTTP/1.1 403 Forbidden\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{\"status\":\"locked\",\"message\":\"Mode Ujian aktif: Tugas Anda telah dikumpulkan sebelumnya dan telah dikunci. Pengumpulan ulang tidak diizinkan.\"}";
                stream.write_all(resp.as_bytes())?;
                return Ok(());
            }

            let rev = match existing_pos {
                Some(pos) => subs[pos].revision + 1,
                None => 1,
            };

            let save_dir = save_dir_ref.lock().unwrap().clone();
            let file_path = std::path::PathBuf::from(&save_dir).join(&safe_filename);
            let _ = fs::write(&file_path, &s_content);

            if let Some(pdf_b64) = payload.get("pdf_base64").and_then(|v| v.as_str()) {
                if let Ok(pdf_bytes) = base64::engine::general_purpose::STANDARD.decode(pdf_b64) {
                    let pdf_filename = if safe_filename.ends_with(".smrl") {
                        format!("{}.pdf", &safe_filename[..safe_filename.len() - 5])
                    } else {
                        format!("{}.pdf", safe_filename)
                    };
                    let pdf_path = std::path::PathBuf::from(&save_dir).join(&pdf_filename);
                    let _ = fs::write(&pdf_path, &pdf_bytes);
                }
            }

            let now = std::time::SystemTime::now();
            let dur = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let secs = dur.as_secs() + 7 * 3600; // UTC+7 WIB
            let hours = (secs / 3600) % 24;
            let mins = (secs / 60) % 60;
            let secs_rem = secs % 60;
            let time_str = format!("{:02}:{:02}:{:02} WIB", hours, mins, secs_rem);

            let rec = SubmissionRecord {
                student_name: s_name,
                presensi_no: p_no,
                filename: safe_filename,
                file_path: file_path.to_string_lossy().to_string(),
                timestamp: time_str,
                content: s_content,
                revision: rev,
            };

            if let Some(pos) = existing_pos {
                subs[pos] = rec;
            } else {
                subs.push(rec);
            }

            let msg = if is_exam {
                "Tugas berhasil dikumpulkan dan dikunci untuk penilaian ujian."
            } else if rev > 1 {
                "Tugas perbaikan (revisi) berhasil diterima guru."
            } else {
                "Tugas berhasil diterima guru."
            };

            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{{\"status\":\"ok\",\"message\":\"{}\",\"revision\":{},\"exam_mode\":{}}}",
                msg, rev, is_exam
            );
            stream.write_all(resp.as_bytes())?;
            return Ok(());
        }
    }

    let resp = "HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\nNot Found";
    stream.write_all(resp.as_bytes())?;
    Ok(())
}

#[tauri::command]
fn start_classroom_server(
    task_name: String,
    circuit_content: String,
    state: tauri::State<AppClassroomState>,
) -> Result<ClassroomServerInfo, String> {
    if state.running.load(Ordering::SeqCst) {
        *state.task_json.lock().unwrap() = circuit_content.clone();
        if let Some(info) = state.server_info.lock().unwrap().clone() {
            return Ok(info);
        }
    }

    let ip = get_local_ip();
    let port_start = 8765;
    let mut listener_opt = None;
    let mut bound_port = port_start;

    for p in port_start..port_start + 20 {
        if let Ok(l) = TcpListener::bind(("0.0.0.0", p)) {
            listener_opt = Some(l);
            bound_port = p;
            break;
        }
    }

    let listener = match listener_opt {
        Some(l) => l,
        None => return Err("Tidak dapat membuka port jaringan (8765-8785)".to_string()),
    };

    let target_dir = get_default_save_dir(&task_name);
    let _ = fs::create_dir_all(&target_dir);
    let dir_str = target_dir.to_string_lossy().to_string();

    let server_url = format!("http://{}:{}/task.smrl", ip, bound_port);
    let is_exam = state.exam_mode.load(Ordering::SeqCst);
    let info = ClassroomServerInfo {
        ip: ip.clone(),
        port: bound_port,
        url: server_url,
        task_name: task_name.clone(),
        save_dir: dir_str.clone(),
        exam_mode: is_exam,
    };

    *state.task_json.lock().unwrap() = circuit_content;
    *state.save_dir.lock().unwrap() = dir_str;
    *state.server_info.lock().unwrap() = Some(info.clone());
    *state.active_port.lock().unwrap() = bound_port;
    state.submissions.lock().unwrap().clear();
    state.running.store(true, Ordering::SeqCst);

    let running_flag = state.running.clone();
    let task_json_ref = state.task_json.clone();
    let submissions_ref = state.submissions.clone();
    let save_dir_ref = state.save_dir.clone();
    let exam_mode_ref = state.exam_mode.clone();

    thread::spawn(move || {
        let _ = listener.set_nonblocking(false);
        while running_flag.load(Ordering::SeqCst) {
            if let Ok((mut stream, _)) = listener.accept() {
                if !running_flag.load(Ordering::SeqCst) {
                    break;
                }
                let t_json = task_json_ref.clone();
                let subs = submissions_ref.clone();
                let s_dir = save_dir_ref.clone();
                let e_mode = exam_mode_ref.clone();

                thread::spawn(move || {
                    let _ = handle_http_client(&mut stream, t_json, subs, s_dir, e_mode);
                });
            }
        }
    });

    Ok(info)
}

#[tauri::command]
fn stop_classroom_server(state: tauri::State<AppClassroomState>) -> Result<(), String> {
    if state.running.load(Ordering::SeqCst) {
        state.running.store(false, Ordering::SeqCst);
        let port = *state.active_port.lock().unwrap();
        let _ = TcpStream::connect(("127.0.0.1", port));
        *state.server_info.lock().unwrap() = None;
    }
    state.submissions.lock().unwrap().clear();
    Ok(())
}

#[tauri::command]
fn get_classroom_status(
    state: tauri::State<AppClassroomState>,
) -> Result<ClassroomStatusResponse, String> {
    let running = state.running.load(Ordering::SeqCst);
    let server_info = state.server_info.lock().unwrap().clone();
    let submissions = state.submissions.lock().unwrap().clone();
    let exam_mode = state.exam_mode.load(Ordering::SeqCst);
    Ok(ClassroomStatusResponse {
        running,
        server_info,
        submissions,
        exam_mode,
    })
}

#[tauri::command]
fn set_classroom_exam_mode(
    exam_mode: bool,
    state: tauri::State<AppClassroomState>,
) -> Result<bool, String> {
    state.exam_mode.store(exam_mode, Ordering::SeqCst);
    if let Some(info) = state.server_info.lock().unwrap().as_mut() {
        info.exam_mode = exam_mode;
    }
    Ok(exam_mode)
}

#[tauri::command]
fn open_classroom_folder(state: tauri::State<AppClassroomState>) -> Result<(), String> {
    let dir = state.save_dir.lock().unwrap().clone();
    if !dir.is_empty() {
        let p = std::path::Path::new(&dir);
        let _ = std::fs::create_dir_all(p);
        let _ = std::process::Command::new("explorer").arg(p).spawn();
    }
    Ok(())
}

#[tauri::command]
fn select_classroom_folder(
    state: tauri::State<AppClassroomState>,
) -> Result<Option<String>, String> {
    let current_dir = state.save_dir.lock().unwrap().clone();
    let mut dialog = rfd::FileDialog::new().set_title("Pilih Folder Pengumpulan Tugas Siswa");
    if !current_dir.is_empty() {
        let p = std::path::Path::new(&current_dir);
        if p.exists() {
            dialog = dialog.set_directory(p);
        }
    }

    if let Some(path) = dialog.pick_folder() {
        let path_str = path.to_string_lossy().to_string();
        *state.save_dir.lock().unwrap() = path_str.clone();
        if let Some(info) = state.server_info.lock().unwrap().as_mut() {
            info.save_dir = path_str.clone();
        }
        Ok(Some(path_str))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn get_app_info() -> SystemAppInfo {
    SystemAppInfo {
        name: "Mashimurelay".to_string(),
        version: "1.0.0".to_string(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

#[tauri::command]
fn save_circuit_file(
    default_filename: String,
    content: String,
    extension: String,
    filter_description: String,
) -> Result<Option<String>, String> {
    let dialog = rfd::FileDialog::new()
        .set_file_name(&default_filename)
        .add_filter(&filter_description, &[&extension]);

    if let Some(path) = dialog.save_file() {
        fs::write(&path, content).map_err(|e| format!("Gagal menyimpan file: {}", e))?;
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn open_circuit_file(
    filter_description: String,
    extensions: Vec<String>,
) -> Result<Option<OpenFileResponse>, String> {
    let ext_slices: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
    let dialog = rfd::FileDialog::new().add_filter(&filter_description, &ext_slices);

    if let Some(path) = dialog.pick_file() {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("Gagal membaca file: {}", e))?;
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "circuit.smrl".to_string());
        Ok(Some(OpenFileResponse {
            file_name,
            file_path: path.to_string_lossy().to_string(),
            content,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn save_image_file(default_filename: String, base64_png: String) -> Result<Option<String>, String> {
    let dialog = rfd::FileDialog::new()
        .set_file_name(&default_filename)
        .add_filter("Gambar Skematik PNG (*.png)", &["png"]);

    if let Some(path) = dialog.save_file() {
        let pure_b64 = if let Some(idx) = base64_png.find(',') {
            &base64_png[idx + 1..]
        } else {
            &base64_png
        };
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(pure_b64.trim())
            .map_err(|e| format!("Format Base64 tidak valid: {}", e))?;
        fs::write(&path, bytes).map_err(|e| format!("Gagal menyimpan gambar: {}", e))?;
        Ok(Some(path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn get_cli_file_arg() -> Option<OpenFileResponse> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let potential_path = &args[1];
        let p = std::path::Path::new(potential_path);
        if p.exists() && p.is_file() {
            if let Ok(content) = fs::read_to_string(p) {
                let file_name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "circuit.smrl".to_string());
                return Some(OpenFileResponse {
                    file_name,
                    file_path: potential_path.clone(),
                    content,
                });
            }
        }
    }
    None
}

#[tauri::command]
fn force_close_app(window: tauri::Window) {
    let _ = window.destroy();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let classroom_state = AppClassroomState {
        running: Arc::new(AtomicBool::new(false)),
        server_info: Arc::new(Mutex::new(None)),
        task_json: Arc::new(Mutex::new(String::new())),
        submissions: Arc::new(Mutex::new(Vec::new())),
        save_dir: Arc::new(Mutex::new(String::new())),
        active_port: Arc::new(Mutex::new(8765)),
        exam_mode: Arc::new(AtomicBool::new(false)),
    };

    tauri::Builder::default()
        .manage(classroom_state)
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = tauri::Emitter::emit(window, "mashimu-close-requested", ());
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_app_info,
            save_circuit_file,
            open_circuit_file,
            save_image_file,
            get_cli_file_arg,
            start_classroom_server,
            stop_classroom_server,
            get_classroom_status,
            set_classroom_exam_mode,
            open_classroom_folder,
            select_classroom_folder,
            force_close_app
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
