// Prevents additional console window on Windows
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// TODO: Implementar comandos Tauri y SidecarManager

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
