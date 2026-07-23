mod diff;
mod fs;
mod menu;
mod ollama;
mod settings;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  use tauri::Manager;

  let app = tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .manage(ollama::client::OllamaClient::new())
    .manage(ollama::process::OllamaProcess::new())
    .invoke_handler(tauri::generate_handler![
      fs::document::read_document,
      fs::document::write_document,
      fs::project::scan_project,
      fs::image::read_image_base64,
      ollama::client::ollama_detect,
      ollama::client::ollama_list_models,
      ollama::client::ollama_supports_thinking,
      ollama::client::ollama_supports_vision,
      ollama::client::ollama_get_model_info,
      ollama::client::generate_edit,
      ollama::client::cancel_generation,
      diff::engine::diff_documents,
      settings::store::get_settings,
      settings::store::set_settings,
      settings::keychain::save_web_search_api_key,
      settings::keychain::has_web_search_api_key,
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      menu::setup(app)?;

      let app_handle = app.handle().clone();
      tauri::async_runtime::spawn(async move {
        let client = app_handle.state::<ollama::client::OllamaClient>();
        let process = app_handle.state::<ollama::process::OllamaProcess>();
        process.ensure_running(&client).await;
      });

      Ok(())
    })
    .build(tauri::generate_context!())
    .expect("error while building tauri application");

  app.run(|app_handle, event| {
    // `ExitRequested` doesn't reliably fire for every quit path (observed: it's skipped
    // entirely when quitting via Cmd+Q / the Quit menu item, which goes straight to `Exit`) —
    // `Exit` is the one event guaranteed to fire last regardless of how the app is told to
    // quit, so that's the reliable place to stop a process we own. `stop_if_owned` is a no-op
    // the second time (the child is already taken), so handling both is harmless.
    if matches!(
      event,
      tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
    ) {
      app_handle
        .state::<ollama::process::OllamaProcess>()
        .stop_if_owned();
    }
  });
}
