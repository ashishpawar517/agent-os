use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = tauri::WindowBuilder::new(app, "main", tauri::WindowUrl::App("index.html".into()))
                .title("Agent OS")
                .inner_size(1200.0, 800.0)
                .min_inner_size(800.0, 600.0)
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}