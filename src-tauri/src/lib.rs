// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use notify::{Error, Event, RecommendedWatcher, RecursiveMode, Watcher, EventKind};
use std::thread;
use std::fs;
use std::path::PathBuf;
use std::{path::Path};
use async_std::task;
use tauri::{Manager, AppHandle};
use once_cell::sync::OnceCell;
use tauri::Emitter;
use rusqlite::{Connection, Result};
use std::process::Command;
use tauri::path::{BaseDirectory};

static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();
// const FILE_PATH_VN: &str = "/tmp/tauri_vn";
const FILE_PATH_EN: &str = "/tmp/tauri_en";

fn filter_alphabets(word: &str) -> String {
    word.chars().filter(|c| c.is_alphabetic()).collect()
}


fn connect_sqlite(text_en: &str, db_path:  &str)  -> Vec<String> {
    let conn = Connection::open(db_path).unwrap();
    let words: Vec<String> = text_en.split_whitespace()
        .map(filter_alphabets)
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.len() == 5 {
                let tmp: String = s.chars().take(3).collect();
                tmp + &"%"
            }  else if s.len() >= 6 {
                let tmp: String = s.chars().take(s.len() - 3).collect();
                tmp + &"%"
            } else {
                let tmp: String = s.to_string();
                tmp
            }
        })
        .collect();
    let conditions: Vec<String> = words.iter().map(|_| "sfld LIKE ?".to_string()).collect();
    let query = format!(
        "SELECT id, sfld FROM notes WHERE {}",
        conditions.join(" OR ")
    );

    let mut stmt = conn.prepare(&query).unwrap();

    let rows: Vec<String> = stmt
        .query_map(rusqlite::params_from_iter(words.iter()), |row| {
            row.get::<_, String>(1) // Lấy giá trị cột thứ 2 (index 1)
        }).unwrap()
        .collect::<Result<Vec<_>, _>>().unwrap();
    return rows;
    // let mut stmt = conn.prepare(query).unwrap();

    // let rows = stmt.query_map(rusqlite::params![words[0], words[1]], |row| {
    //     let value: u64 = row.get(0).unwrap();
    //     Ok(value)
    // }).unwrap();

    // for row in rows {
    //     println!("{:?}", row.unwrap());
    // }
}
fn watch_named_pipe() {
     thread::spawn(move || {
        let mut watcher = RecommendedWatcher::new(move |result: Result<Event, Error>| {
            let event = result.unwrap();
            if let EventKind::Modify(notify::event::ModifyKind::Data(_)) = event.kind {
                if let Some(app_handle) = GLOBAL_APP_HANDLE.get() {
                    let content_en = fs::read_to_string(FILE_PATH_EN).unwrap();
                    let content_en = content_en.trim();
                    let anki_path = app_handle.path().resolve("bin/collection.anki2", BaseDirectory::Resource).unwrap();
                    let rows: Vec<String> = connect_sqlite(content_en, &anki_path.display().to_string());
                    app_handle.emit("backend_anki", rows).unwrap();
                    app_handle.emit("backend_text_en", content_en).unwrap();
                }
            }
        },notify::Config::default()).unwrap();
        let _ = watcher.watch(Path::new(FILE_PATH_EN), RecursiveMode::Recursive);

        task::block_on(async {
            loop {
                task::sleep(std::time::Duration::from_secs(2)).await;
            }
        });

    });
}

#[tauri::command]
fn save_setting(data: serde_json::Value, app_handle: tauri::AppHandle) {
    let resource_path = app_handle.path().resolve("bin/my_config.json", BaseDirectory::Resource).unwrap();
    let json_string = serde_json::to_string_pretty(&data).map_err(|e| format!("Lỗi khi chuyển JSON thành chuỗi: {}", e)).unwrap();
    fs::write(resource_path, json_string).expect("Không thể ghi file");
}

#[tauri::command]
fn load_setting(app_handle: tauri::AppHandle) -> serde_json::Value {
    let resource_path = app_handle.path().resolve("bin/my_config.json", BaseDirectory::Resource).unwrap();
    let content = fs::read_to_string(resource_path).expect("Không thể đọc file");
    let data: serde_json::Value = serde_json::from_str(&content).unwrap();
    if data["always-on-top"] == 1 {
        let window = app_handle.get_webview_window("main").unwrap();
        window.set_always_on_top(true).unwrap(); // Bật Always On Top
    } else {
        let window = app_handle.get_webview_window("main").unwrap();
        window.set_always_on_top(false).unwrap(); // Bật Always On Top
    }
    return data;
}


#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    fs::read_to_string(path_buf).map_err(|e| e.to_string())
}

#[tauri::command]
fn send_to_anki(text: String) {
    let script = format!(r#"
        tell application "Anki" to activate
        delay 0.1
        tell application "System Events"
            keystroke "f" using command down
            keystroke "*word:{}*"
            key code 36 -- Enter
        end tell
    "#, text);
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            watch_named_pipe();
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![read_file, save_setting, load_setting, send_to_anki])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| match event {
            tauri::RunEvent::Ready => { 
                GLOBAL_APP_HANDLE.set(_app_handle.clone()).expect("Failed to set global app handle");
            }
            _ => {}
        });
}
