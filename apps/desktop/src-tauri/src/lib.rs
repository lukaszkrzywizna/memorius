mod agent;
mod commands;
mod commits;
mod icons;
mod recording;
mod reminder;
mod setup;
mod shell;
mod tracker;
mod updates;
mod vault;

use std::sync::{Arc, Mutex};

use memorius_core::store::Store;
use tauri::{App, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

use crate::agent::AgentRunner;
use crate::commands::{Database, Git, Reader};
use crate::commits::Commits;
use crate::recording::Recording;
use crate::reminder::Reminder;
use crate::tracker::Tracker;
use crate::vault::Vault;

/// Passed by the login item so Memorius starts in the menu bar only.
const HIDDEN_ARG: &str = "--hidden";

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = shell::show_main(app);
        }))
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![HIDDEN_ARG]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .register_asynchronous_uri_scheme_protocol(icons::SCHEME, icons::respond)
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            commands::day_timeline,
            commands::recorder_status,
            commands::pause_recording,
            commands::resume_recording,
            commands::request_title_access,
            commands::show_main_window,
            commands::quit_app,
            commands::restart_app,
            commands::rules,
            commands::recent_apps,
            commands::add_rule,
            commands::remove_rule,
            commands::log_queue,
            commands::approve_proposal,
            commands::set_sessions_private,
            commands::reject_proposal,
            commands::undo_proposal,
            commands::week_summary,
            commands::day_summaries,
            commands::rule_matches,
            commands::search,
            commands::prompt_text,
            agent::agent_settings,
            agent::save_agent_settings,
            agent::agent_runs,
            agent::start_agent_run,
            agent::stop_agent_run,
            agent::agent_prompt,
            tracker::tracker_status,
            tracker::connect_tracker,
            tracker::disconnect_tracker,
            tracker::sync_tracker,
            tracker::log_approved,
            tracker::tracker_work_types,
            reminder::reminder_settings,
            reminder::save_reminder_settings,
            reminder::test_reminder,
            commits::git_status,
            commits::save_git_settings,
            commits::commit_details,
            commits::open_link,
            updates::check_update,
            commands::time_rules,
            commands::save_time_rules,
            agent::agent_auth,
            agent::agent_sign_in,
            agent::install_cli,
            agent::add_to_agent,
            setup::setup_status,
            setup::finish_setup,
            setup::confirm_notifications,
            setup::start_at_login,
            setup::open_system_pane,
            updates::install_update,
            agent::agent_check,
        ])
        .build(tauri::generate_context!())
        .expect("Memorius failed to start the Tauri runtime");

    app.run(|app, event| match event {
        // Closing the last window keeps recording; only "Quit" exits.
        RunEvent::ExitRequested {
            code: None, api, ..
        } => api.prevent_exit(),
        RunEvent::Exit => app.state::<Recording>().stop(),
        RunEvent::WindowEvent {
            label,
            event: WindowEvent::Destroyed,
            ..
        } if label == "main" => shell::on_main_destroyed(app),
        _ => {}
    });
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    let database = data_dir.join("memorius.db");
    let first_run = !database.exists();

    let recording = Recording::start(app.handle().clone(), &database)?;
    app.manage(Reader(Mutex::new(Store::open_read_only(&database)?)));
    app.manage(recording);
    app.manage(Git::default());
    app.manage(AgentRunner::new(&database)?);
    agent::start_schedule(app.handle().clone());
    app.manage(Tracker::new(&database, Arc::new(Vault::new(&database))));
    tracker::start_sync(app.handle().clone());
    app.manage(Reminder::new(&database));
    reminder::start(app.handle().clone());
    app.manage(Commits::new(&database));
    commits::start(app.handle().clone());
    updates::start(app.handle().clone());
    app.manage(Database(database.clone()));
    memorius_sources::import::start(database);
    shell::create_tray(app.handle())?;

    // Register the login item once, and only for installed (release) builds.
    if first_run && !cfg!(debug_assertions) {
        app.autolaunch().enable()?;
    }
    if std::env::args().any(|arg| arg == HIDDEN_ARG) {
        shell::on_main_destroyed(app.handle());
    } else {
        shell::show_main(app.handle())?;
    }
    Ok(())
}
