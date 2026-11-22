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
    #[serde(default)]
    pub has_jobsheet: bool,
    #[serde(default)]
    pub jobsheet_name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClassroomStatusResponse {
    pub running: bool,
    pub server_info: Option<ClassroomServerInfo>,
    pub submissions: Vec<SubmissionRecord>,
    pub exam_mode: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FirewallStatusResponse {
    pub status: String, // "allowed" | "blocked" | "partial" | "unconfigured" | "unknown"
    pub exe_path: String,
    pub detail: String,
}

pub struct AppClassroomState {
    pub running: Arc<AtomicBool>,
    pub server_info: Arc<Mutex<Option<ClassroomServerInfo>>>,
    pub task_json: Arc<Mutex<String>>,
    pub submissions: Arc<Mutex<Vec<SubmissionRecord>>>,
    pub save_dir: Arc<Mutex<String>>,
    pub active_port: Arc<Mutex<u16>>,
    pub exam_mode: Arc<AtomicBool>,
    pub jobsheet_bytes: Arc<Mutex<Option<Vec<u8>>>>,
    pub jobsheet_filename: Arc<Mutex<Option<String>>>,
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
    jobsheet_bytes_ref: Arc<Mutex<Option<Vec<u8>>>>,
    jobsheet_filename_ref: Arc<Mutex<Option<String>>>,
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
        let resp = "HTTP/1.1 204 No Content\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Expose-Headers: X-Exam-Mode, X-Has-Jobsheet, X-Jobsheet-Name\r\nConnection: close\r\n\r\n";
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "GET" && (path == "/task.smrl" || path == "/") {
        let content = task_json_ref.lock().unwrap().clone();
        let content_bytes = content.as_bytes();
        let js_name_opt = jobsheet_filename_ref.lock().unwrap().clone();
        let has_js = jobsheet_bytes_ref.lock().unwrap().is_some();
        let js_header_val = if has_js { "true" } else { "false" };
        let js_name_val = js_name_opt.unwrap_or_default();
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Expose-Headers: X-Exam-Mode, X-Has-Jobsheet, X-Jobsheet-Name\r\nX-Exam-Mode: {}\r\nX-Has-Jobsheet: {}\r\nX-Jobsheet-Name: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            is_exam,
            js_header_val,
            js_name_val,
            content_bytes.len(),
            content
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "GET" && path == "/info" {
        let js_name_opt = jobsheet_filename_ref.lock().unwrap().clone();
        let has_js = jobsheet_bytes_ref.lock().unwrap().is_some();
        let js_name_val = js_name_opt.unwrap_or_default();
        let resp_json = format!(
            "{{\"status\":\"ok\",\"exam_mode\":{},\"has_jobsheet\":{},\"jobsheet_name\":\"{}\"}}",
            is_exam,
            has_js,
            js_name_val.replace('"', "\\\"")
        );
        let resp = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Expose-Headers: X-Exam-Mode, X-Has-Jobsheet, X-Jobsheet-Name\r\nX-Exam-Mode: {}\r\nX-Has-Jobsheet: {}\r\nX-Jobsheet-Name: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            is_exam,
            if has_js { "true" } else { "false" },
            js_name_val,
            resp_json.len(),
            resp_json
        );
        stream.write_all(resp.as_bytes())?;
        return Ok(());
    }

    if method == "GET" && (path == "/jobsheet.pdf" || path == "/jobsheet") {
        let js_bytes_opt = jobsheet_bytes_ref.lock().unwrap().clone();
        let js_name_opt = jobsheet_filename_ref.lock().unwrap().clone();
        match js_bytes_opt {
            Some(bytes) => {
                let filename = js_name_opt.unwrap_or_else(|| "jobsheet.pdf".to_string());
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\nContent-Disposition: inline; filename=\"{}\"\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Expose-Headers: X-Exam-Mode, X-Has-Jobsheet, X-Jobsheet-Name\r\nX-Has-Jobsheet: true\r\nX-Jobsheet-Name: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
                    bytes.len(),
                    filename,
                    filename
                );
                stream.write_all(header.as_bytes())?;
                stream.write_all(&bytes)?;
                return Ok(());
            }
            None => {
                let resp_body = b"{\"error\":\"Jobsheet tidak tersedia\"}";
                let resp = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: X-Exam-Mode, X-Has-Jobsheet\r\nX-Has-Jobsheet: false\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    resp_body.len()
                );
                stream.write_all(resp.as_bytes())?;
                stream.write_all(resp_body)?;
                return Ok(());
            }
        }
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
    jobsheet_base64: Option<String>,
    jobsheet_filename: Option<String>,
    state: tauri::State<AppClassroomState>,
) -> Result<ClassroomServerInfo, String> {
    let mut js_bytes: Option<Vec<u8>> = None;
    let mut js_name: Option<String> = None;

    if let Some(ref b64) = jobsheet_base64 {
        if !b64.trim().is_empty() {
            let pure_b64 = match b64.find(',') {
                Some(idx) if b64[..idx].contains("base64") => &b64[idx + 1..],
                _ => b64.trim(),
            };
            if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(pure_b64) {
                if !decoded.is_empty() && decoded.len() <= 25 * 1024 * 1024 {
                    let raw_name = jobsheet_filename.as_deref().unwrap_or("jobsheet.pdf");
                    let clean_name = raw_name.rsplit(|c| c == '/' || c == '\\').next().unwrap_or("jobsheet.pdf");
                    let safe_name = if clean_name.to_ascii_lowercase().ends_with(".pdf") {
                        clean_name.to_string()
                    } else {
                        format!("{}.pdf", clean_name)
                    };
                    js_bytes = Some(decoded);
                    js_name = Some(safe_name);
                }
            }
        }
    }

    let has_js = js_bytes.is_some();
    *state.jobsheet_bytes.lock().unwrap() = js_bytes;
    *state.jobsheet_filename.lock().unwrap() = js_name.clone();

    if state.running.load(Ordering::SeqCst) {
        *state.task_json.lock().unwrap() = circuit_content.clone();
        if let Some(mut info) = state.server_info.lock().unwrap().clone() {
            info.has_jobsheet = has_js;
            info.jobsheet_name = js_name;
            *state.server_info.lock().unwrap() = Some(info.clone());
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
        has_jobsheet: has_js,
        jobsheet_name: js_name,
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
    let jobsheet_bytes_ref = state.jobsheet_bytes.clone();
    let jobsheet_filename_ref = state.jobsheet_filename.clone();

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
                let js_b = jobsheet_bytes_ref.clone();
                let js_f = jobsheet_filename_ref.clone();

                thread::spawn(move || {
                    let _ = handle_http_client(&mut stream, t_json, subs, s_dir, e_mode, js_b, js_f);
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
    *state.jobsheet_bytes.lock().unwrap() = None;
    *state.jobsheet_filename.lock().unwrap() = None;
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

fn current_exe_clean_path() -> String {
    let raw = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    raw.strip_prefix(r"\\?\").unwrap_or(&raw).to_string()
}

#[cfg(target_os = "windows")]
fn check_firewall_status_sync() -> FirewallStatusResponse {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let exe_path = current_exe_clean_path();
    if exe_path.is_empty() {
        return FirewallStatusResponse {
            status: "unknown".to_string(),
            exe_path,
            detail: "Lokasi berkas aplikasi tidak terdeteksi.".to_string(),
        };
    }

    let ps_script = concat!(
        "$p = $env:MASHIMU_EXE_PATH.ToLowerInvariant(); ",
        "$hasBlock = $false; $hasPriv = $false; $hasPub = $false; ",
        "try { ",
        "  Get-NetFirewallApplicationFilter -ErrorAction Stop | Where-Object { $_.Program -and $_.Program.ToLowerInvariant() -eq $p } | ForEach-Object { ",
        "    $r = $_ | Get-NetFirewallRule -ErrorAction SilentlyContinue; ",
        "    if ($r -and $r.Enabled -eq 'True' -and $r.Direction -eq 'Inbound') { ",
        "      $prof = $r.Profile.ToString(); ",
        "      if ($r.Action -eq 'Block') { $hasBlock = $true } ",
        "      elseif ($r.Action -eq 'Allow') { ",
        "        if ($prof -match 'Any' -or $prof -match 'Private') { $hasPriv = $true } ",
        "        if ($prof -match 'Any' -or $prof -match 'Public') { $hasPub = $true } ",
        "      } ",
        "    } ",
        "  }; ",
        "  if ($hasBlock) { Write-Output 'BLOCKED' } ",
        "  elseif ($hasPriv -and $hasPub) { Write-Output 'ALLOWED' } ",
        "  elseif ($hasPriv -or $hasPub) { Write-Output 'PARTIAL' } ",
        "  else { Write-Output 'UNCONFIGURED' } ",
        "} catch { Write-Output 'UNKNOWN' }"
    );

    let output = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", ps_script])
        .env("MASHIMU_EXE_PATH", &exe_path)
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let code = match output {
        Ok(out) => String::from_utf8_lossy(&out.stdout).trim().to_ascii_uppercase(),
        Err(_) => "UNKNOWN".to_string(),
    };

    match code.as_str() {
        "BLOCKED" => FirewallStatusResponse {
            status: "blocked".to_string(),
            exe_path,
            detail: "Akses Hotspot terblokir Windows Firewall (kemungkinan sebelumnya menekan Batal). HP siswa tidak dapat terhubung sebelum izin dibuka.".to_string(),
        },
        "ALLOWED" => FirewallStatusResponse {
            status: "allowed".to_string(),
            exe_path,
            detail: "Whitelist Windows Firewall aktif (Private & Public). HP siswa siap terhubung.".to_string(),
        },
        "PARTIAL" => FirewallStatusResponse {
            status: "partial".to_string(),
            exe_path,
            detail: "Izin Firewall baru aktif di sebagian profil jaringan. Pasang whitelist penuh (Private & Public) agar koneksi Hotspot selalu lancar.".to_string(),
        },
        "UNCONFIGURED" => FirewallStatusResponse {
            status: "unconfigured".to_string(),
            exe_path,
            detail: "Whitelist Windows Firewall belum terpasang untuk berkas ini (versi Portable). Klik tombol di samping agar koneksi siswa bebas hambatan.".to_string(),
        },
        _ => FirewallStatusResponse {
            status: "unknown".to_string(),
            exe_path,
            detail: "Tidak dapat membaca status Windows Firewall secara otomatis.".to_string(),
        },
    }
}

#[cfg(not(target_os = "windows"))]
fn check_firewall_status_sync() -> FirewallStatusResponse {
    FirewallStatusResponse {
        status: "allowed".to_string(),
        exe_path: current_exe_clean_path(),
        detail: "Firewall Windows tidak berlaku pada sistem ini.".to_string(),
    }
}

#[tauri::command]
async fn check_classroom_firewall_status() -> Result<FirewallStatusResponse, String> {
    tauri::async_runtime::spawn_blocking(check_firewall_status_sync)
        .await
        .map_err(|e| format!("Gagal memeriksa firewall: {}", e))
}

#[cfg(target_os = "windows")]
fn allow_classroom_firewall_sync() -> Result<FirewallStatusResponse, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let exe_path = current_exe_clean_path();
    if exe_path.is_empty() {
        return Err("Lokasi aplikasi Mashimurelay tidak ditemukan.".to_string());
    }

    let escaped_exe = exe_path.replace('\'', "''");
    let script_body = format!(
        "$target = '{exe}'.ToLowerInvariant()\r\n\
         Get-NetFirewallApplicationFilter -ErrorAction SilentlyContinue | Where-Object {{ $_.Program -and $_.Program.ToLowerInvariant() -eq $target }} | ForEach-Object {{\r\n\
           $r = $_ | Get-NetFirewallRule -ErrorAction SilentlyContinue\r\n\
           if ($r -and $r.Direction -eq 'Inbound' -and ($r.Action -eq 'Block' -or $r.DisplayName -eq 'Mashimurelay Hotspot Kelas')) {{\r\n\
             Remove-NetFirewallRule -Name $r.Name -ErrorAction SilentlyContinue\r\n\
           }}\r\n\
         }}\r\n\
         New-NetFirewallRule -DisplayName 'Mashimurelay Hotspot Kelas' -Direction Inbound -Action Allow -Program '{exe}' -Protocol TCP -LocalPort 8765-8785 -Profile Private,Public -Enabled True -Description 'Mengizinkan koneksi offline LAN Hotspot Kelas Mashimurelay (port 8765-8785)' -ErrorAction Stop | Out-Null\r\n",
        exe = escaped_exe
    );

    let temp_script = std::env::temp_dir().join(format!("mashimurelay_fw_{}.ps1", std::process::id()));
    fs::write(&temp_script, script_body)
        .map_err(|e| format!("Gagal menyiapkan skrip firewall: {}", e))?;

    let escaped_script = temp_script.to_string_lossy().replace('\'', "''");
    let elevate_cmd = format!(
        "try {{ $p = Start-Process -FilePath 'powershell.exe' -ArgumentList @('-NoProfile','-ExecutionPolicy','Bypass','-WindowStyle','Hidden','-File','{script}') -Verb RunAs -WindowStyle Hidden -Wait -PassThru; exit $p.ExitCode }} catch {{ exit 1223 }}",
        script = escaped_script
    );

    let status = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &elevate_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    let _ = fs::remove_file(&temp_script);

    match status {
        Ok(s) if s.code() == Some(1223) => {
            Err("Konfirmasi izin Administrator (UAC) dibatalkan oleh pengguna.".to_string())
        }
        Ok(s) if s.success() => Ok(check_firewall_status_sync()),
        Ok(s) => Err(format!("Konfigurasi Windows Firewall gagal (kode {:?}).", s.code())),
        Err(e) => Err(format!("Gagal menjalankan konfigurasi Firewall: {}", e)),
    }
}

#[cfg(not(target_os = "windows"))]
fn allow_classroom_firewall_sync() -> Result<FirewallStatusResponse, String> {
    Ok(check_firewall_status_sync())
}

#[tauri::command]
async fn allow_classroom_firewall() -> Result<FirewallStatusResponse, String> {
    tauri::async_runtime::spawn_blocking(allow_classroom_firewall_sync)
        .await
        .map_err(|e| format!("Gagal memproses firewall: {}", e))?
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
        jobsheet_bytes: Arc::new(Mutex::new(None)),
        jobsheet_filename: Arc::new(Mutex::new(None)),
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
            check_classroom_firewall_status,
            allow_classroom_firewall,
            force_close_app
        ])
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
