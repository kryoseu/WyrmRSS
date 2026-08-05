mod backend;
mod error;

use tracing::error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = backend::start_backend(&app_handle).await {
                    error!("failed to start embedded backend: {err}");
                }
            });

            // A terminal Ctrl+C sends SIGINT straight to the process with no
            // window-close event, so it bypasses the RunEvent::Exit handler
            // below unless we catch it explicitly here.
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if tokio::signal::ctrl_c().await.is_ok() {
                    backend::shutdown().await;
                    app_handle.exit(0);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![backend::server_info])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                tauri::async_runtime::block_on(backend::shutdown());
            }
        });
}
