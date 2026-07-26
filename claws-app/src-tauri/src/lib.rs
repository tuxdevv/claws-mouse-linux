mod hid;

use serde::Serialize;

#[derive(Serialize)]
struct ProfileOut {
    polling: u8,
    stage: u8,
    table: [u16; 6],
}

#[tauri::command]
fn get_info() -> Result<String, String> {
    let dev = hid::find_device()?;
    hid::get_info(&dev)
}

#[tauri::command]
fn get_profile() -> Result<ProfileOut, String> {
    let dev = hid::find_device()?;
    let p = hid::get_profile(&dev)?;
    Ok(ProfileOut { polling: p.polling, stage: p.stage, table: p.table })
}

#[tauri::command]
fn set_dpi_stage(stage: u8) -> Result<bool, String> {
    let dev = hid::find_device()?;
    hid::set_dpi_stage(&dev, stage)
}

#[tauri::command]
fn set_dpi_value(stage: u8, value: u16) -> Result<bool, String> {
    let dev = hid::find_device()?;
    hid::set_dpi_value(&dev, stage, value)
}

#[tauri::command]
fn set_polling(level: u8) -> Result<bool, String> {
    let dev = hid::find_device()?;
    hid::set_polling(&dev, level)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_info,
            get_profile,
            set_dpi_stage,
            set_dpi_value,
            set_polling
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
