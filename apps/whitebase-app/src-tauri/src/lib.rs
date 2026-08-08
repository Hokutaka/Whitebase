mod benchmark;
mod scalar_f64;

use benchmark::run_benchmark;
use scalar_f64::observe_add_scalar_f64;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            run_benchmark,
            observe_add_scalar_f64,
        ])
        .run(tauri::generate_context!())
        .expect("error while running White Base");
}
