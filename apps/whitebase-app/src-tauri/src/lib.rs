mod benchmark;

use benchmark::run_add_f32_benchmark;

#[tauri::command]
fn add(left: i32, right: i32) -> i32 {
    whitebase_rust_backend::add(left, right)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![add, run_add_f32_benchmark,])
        .run(tauri::generate_context!())
        .expect("error while running White Base");
}
