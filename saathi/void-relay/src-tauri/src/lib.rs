pub mod ai;
pub mod audio;
pub mod bluetooth;
pub mod bridge;
pub mod call;
pub mod commands;
pub mod config;
pub mod models;
pub mod telegram;

use std::sync::Arc;
use parking_lot::RwLock;
use tauri::Manager;

use crate::bridge::ws_server::WsServer;
use crate::call::orchestrator::CallOrchestrator;
use crate::config::settings::AppSettings;
use crate::telegram::bot::TelegramBot;
use crate::ai::engine_router::EngineRouter;
use crate::models::model_manager::ModelManager;

/// Shared application state accessible from all Tauri commands
pub struct AppState {
    pub settings: Arc<RwLock<AppSettings>>,
    pub ws_server: Arc<WsServer>,
    pub orchestrator: Arc<CallOrchestrator>,
    pub engine_router: Arc<RwLock<EngineRouter>>,
    pub model_manager: Arc<ModelManager>,
    pub telegram_bot: Arc<RwLock<Option<TelegramBot>>>,
    pub db: Arc<parking_lot::Mutex<rusqlite::Connection>>,
}

pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .init();

    log::info!("VoidRelay starting up...");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .setup(|app| {
            let app_handle = app.handle().clone();
            let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
                directories::ProjectDirs::from("com", "voidstack", "VoidRelay")
                    .map(|p| p.data_dir().to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
            });
            std::fs::create_dir_all(&data_dir).ok();

            // Initialize SQLite database
            let db_path = data_dir.join("voidrelay.db");
            let db = rusqlite::Connection::open(&db_path)
                .expect("Failed to open database");
            call::history::init_db(&db);
            let db = Arc::new(parking_lot::Mutex::new(db));

            // Load settings
            let settings = AppSettings::load(&data_dir);
            let settings = Arc::new(RwLock::new(settings));

            // Initialize model manager
            let model_manager = Arc::new(ModelManager::new(data_dir.clone()));

            // Initialize engine router
            let engine_router = EngineRouter::new(settings.clone(), model_manager.clone());
            let engine_router = Arc::new(RwLock::new(engine_router));

            // Initialize WebSocket server
            let ws_server = Arc::new(WsServer::new());

            // Initialize call orchestrator
            let orchestrator = Arc::new(CallOrchestrator::new(
                app_handle.clone(),
                ws_server.clone(),
                engine_router.clone(),
                db.clone(),
                settings.clone(),
            ));

            let state = AppState {
                settings: settings.clone(),
                ws_server: ws_server.clone(),
                orchestrator: orchestrator.clone(),
                engine_router,
                model_manager,
                telegram_bot: Arc::new(RwLock::new(None)),
                db,
            };

            app.manage(state);

            // Start WebSocket server in background
            let ws = ws_server.clone();
            let orch = orchestrator.clone();
            let app_h = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let port = 7331u16;
                log::info!("Starting WebSocket server on 0.0.0.0:{}", port);
                if let Err(e) = ws.start(port, orch, app_h).await {
                    log::error!("WebSocket server error: {}", e);
                }
            });

            // Start Telegram bot if configured
            let s = settings.clone();
            let ws2 = ws_server.clone();
            let orch2 = orchestrator.clone();
            let app_h2 = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let (token, chat_id) = {
                    let cfg = s.read();
                    if !cfg.telegram_bot_token.is_empty() && !cfg.telegram_chat_id.is_empty() {
                        (Some(cfg.telegram_bot_token.clone()), Some(cfg.telegram_chat_id.clone()))
                    } else {
                        (None, None)
                    }
                };

                if let (Some(token), Some(chat_id)) = (token, chat_id) {
                    let mut bot = TelegramBot::new(token, chat_id);
                    log::info!("Starting Telegram bot polling...");
                    bot.start_polling(ws2, orch2, s, app_h2).await;
                } else {
                    log::info!("Telegram bot not configured, skipping...");
                }
            });

            // Start connectivity checker
            let s2 = settings.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    let online = reqwest::Client::new()
                        .get("https://api.groq.com")
                        .timeout(std::time::Duration::from_secs(5))
                        .send()
                        .await
                        .is_ok();
                    {
                        let mut cfg = s2.write();
                        cfg.online_available = online;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::audio_cmds::list_audio_devices,
            commands::audio_cmds::get_audio_diagnostics,
            commands::audio_cmds::play_test_tone,
            commands::audio_cmds::record_and_playback_test,
            commands::audio_cmds::run_bt_diagnostics_dump,
            commands::audio_cmds::auto_route_audio,
            commands::config_cmds::get_settings,
            commands::config_cmds::save_settings,
            commands::config_cmds::get_network_ips,
            commands::config_cmds::test_groq_key,
            commands::config_cmds::test_sarvam_key,
            commands::config_cmds::test_telegram_bot,
            commands::model_cmds::list_available_models,
            commands::model_cmds::list_downloaded_models,
            commands::model_cmds::download_model,
            commands::model_cmds::cancel_download,
            commands::model_cmds::delete_model,
            commands::model_cmds::switch_model,
            commands::model_cmds::get_active_models,
            commands::model_cmds::get_recommendations,
            commands::model_cmds::verify_model_integrity,
            commands::model_cmds::get_device_profile,
            commands::call_cmds::initiate_call,
            commands::call_cmds::hangup_call,
            commands::call_cmds::take_over_call,
            commands::call_cmds::get_call_state,
            commands::call_cmds::get_call_history,
            commands::call_cmds::export_call_history,
            commands::call_cmds::clear_call_history,
            commands::call_cmds::say_text,
            commands::call_cmds::process_browser_audio,
            commands::bluetooth_cmds::scan_bluetooth,
            commands::bluetooth_cmds::connect_bluetooth,
            commands::bluetooth_cmds::disconnect_bluetooth,
            commands::bluetooth_cmds::get_bluetooth_state,
            commands::log_cmds::get_logs,
            commands::log_cmds::export_logs,
            commands::log_cmds::get_connected_phones,
        ])
        .run(tauri::generate_context!())
        .expect("error while running VoidRelay");
}
