mod backend;
mod error;
mod youtube;

use tracing::error;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Built programmatically (not via tauri.conf.json) so on_navigation
            // can send external links to the system browser instead of
            // navigating the app's own window away, with no way back.
            tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("WyrmRSS")
            .inner_size(1200.0, 800.0)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .fullscreen(false)
            .on_navigation(|url| {
                // Also covers nested frames: the /youtube-embed proxy iframe,
                // and the real youtube.com iframe nested inside that.
                let is_internal = url.scheme() == "tauri"
                    || url.host_str() == Some("tauri.localhost")
                    || url.host_str() == Some("127.0.0.1")
                    || (url.host_str() == Some("www.youtube.com")
                        && url.path().starts_with("/embed/"))
                    || (cfg!(debug_assertions) && url.host_str() == Some("localhost"));
                if !is_internal {
                    let _ = tauri_plugin_opener::open_url(url.as_str(), None::<&str>);
                }
                is_internal
            })
            .build()?;

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
