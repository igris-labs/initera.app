mod api_contract;
mod content;
mod cycle_time;
mod cycles;
mod drafts;
mod issues;
mod labels;
#[cfg(target_os = "macos")]
mod macos_quit;
mod media;
mod mutations;
mod options;
mod planning;
mod portability;
mod portable_archive;
mod portable_json;
mod search;
mod shutdown;
mod storage;
mod time;
mod views;
mod workflow;

use tauri::{Emitter, Manager};

#[cfg(target_os = "macos")]
fn install_sleep_notifications(app: &tauri::AppHandle) {
    use block2::RcBlock;
    use objc2_app_kit::{
        NSWorkspace, NSWorkspaceDidWakeNotification, NSWorkspaceWillSleepNotification,
    };

    let workspace = NSWorkspace::sharedWorkspace();
    let center = workspace.notificationCenter();
    let sleep_app = app.clone();
    let sleep = RcBlock::new(move |_| {
        let result = storage::document_data_root(&sleep_app)
            .and_then(|root| storage::open_database(&root))
            .and_then(|mut connection| time::pause_on_sleep(&mut connection));
        match result {
            Ok(count) if count > 0 => {
                let _ = sleep_app.emit("timers-paused-after-sleep", count);
            }
            Err(error) => eprintln!("Cannot pause timers on suspend: {error}"),
            _ => {}
        }
    });
    let wake_app = app.clone();
    let wake = RcBlock::new(move |_| {
        let app = wake_app.clone();
        std::thread::spawn(move || loop {
            let result = storage::document_data_root(&app)
                .and_then(|root| storage::open_database(&root))
                .and_then(|mut connection| time::resume_after_sleep(&mut connection));
            match result {
                Ok(()) => {
                    let _ = app.emit("timer-recovery-changed", ());
                    break;
                }
                Err(error) => {
                    eprintln!("Cannot reconcile timers on wake; retrying: {error}");
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
            }
        });
    });
    // NSWorkspace uses its own notification center. Retain tokens for the app lifetime.
    unsafe {
        let sleep_token = center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceWillSleepNotification),
            None,
            None,
            &sleep,
        );
        let wake_token = center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceDidWakeNotification),
            None,
            None,
            &wake,
        );
        std::mem::forget(sleep_token);
        std::mem::forget(wake_token);
    }
}

fn flush_timers_before_exit(app: &tauri::AppHandle) -> Result<(), String> {
    let root = storage::document_data_root(app)?;
    let mut connection = storage::open_database(&root)?;
    time::pause_running_on_exit(&mut connection)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let root = storage::document_data_root(&app.handle()).map_err(std::io::Error::other)?;
            std::fs::create_dir_all(root.join("recovery"))?;
            let restore_was_requested = root.join("recovery/restore.requested").exists();
            if let Err(error) = portability::apply_pending_restore(&root) {
                portability::discard_restore_receipt(&root);
                let recovery = root.join("recovery");
                std::fs::write(recovery.join("restore.failed"), &error)?;
                let request = recovery.join("restore.requested");
                if request.exists() {
                    std::fs::remove_file(request)?;
                }
                // A failed swap may have moved the old database into recovery.
                if !root.join("workspace.sqlite").exists()
                    || recovery.join("restore.pending").exists()
                {
                    return Err(std::io::Error::other(error).into());
                }
                let _ = storage::open_database(&root).map_err(std::io::Error::other)?;
            }
            portability::install_restore_receipt(&root, restore_was_requested)
                .map_err(std::io::Error::other)?;
            let mut connection = storage::open_database(&root).map_err(std::io::Error::other)?;
            time::recover_on_start(&mut connection).map_err(std::io::Error::other)?;
            #[cfg(target_os = "macos")]
            {
                macos_quit::install(&app.handle()).map_err(std::io::Error::other)?;
                install_sleep_notifications(&app.handle());
            }
            if !portability::restore_catchup_pending(&root) {
                cycles::reconcile_current_workspace(&mut connection)
                    .map_err(std::io::Error::other)?;
            }
            if let Err(error) = media::cleanup_media(&root, &connection) {
                eprintln!("Cannot clean old media: {error}");
            }
            drop(connection);
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut heartbeat_count = 0_u32;
                loop {
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    let result = storage::open_database(&root).and_then(|mut connection| {
                        let paused = time::heartbeat(&mut connection)?;
                        if !portability::restore_catchup_pending(&root) {
                            cycles::reconcile_current_workspace(&mut connection)?;
                        }
                        heartbeat_count += 1;
                        if heartbeat_count >= 17280 {
                            heartbeat_count = 0;
                            if let Err(error) = media::cleanup_media(&root, &connection) {
                                eprintln!("Cannot clean old media: {error}");
                            }
                        }
                        Ok(paused)
                    });
                    match result {
                        Ok(paused) if paused > 0 => {
                            let _ = handle.emit("timers-paused-after-sleep", paused);
                        }
                        Err(error) => eprintln!("Cannot update timer heartbeat: {error}"),
                        _ => {}
                    }
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            shutdown::storage_finish_close_api,
            api_contract::storage_resolve_issue_entity_ids_api,
            drafts::storage_load_draft_api,
            drafts::storage_save_draft_api,
            drafts::storage_discard_draft_api,
            options::storage_get_workspace_options_api,
            options::storage_set_project_move_preference_api,
            storage::storage_bootstrap_api,
            storage::storage_save_identity_api,
            storage::storage_identity_version_api,
            storage::storage_create_workspace_api,
            storage::storage_switch_workspace_api,
            storage::storage_list_projects_page_api,
            storage::storage_get_project_api,
            storage::storage_get_project_by_name_api,
            storage::storage_list_inactive_projects_api,
            storage::storage_set_project_lifecycle_api,
            storage::storage_preview_project_lifecycle_api,
            storage::storage_list_removed_milestones_api,
            storage::storage_preview_milestone_removal_api,
            storage::storage_set_milestone_archived_api,
            storage::storage_restore_milestone_api,
            storage::storage_create_project_full_api,
            storage::storage_update_project_full_api,
            storage::storage_change_project_resource_api,
            storage::storage_list_project_updates_api,
            storage::storage_create_project_update_api,
            planning::storage_list_planning_nodes_api,
            planning::storage_create_planning_node_api,
            planning::storage_update_planning_node_api,
            planning::storage_reorder_planning_node_api,
            planning::storage_archive_planning_node_api,
            planning::storage_restore_archived_planning_node_api,
            planning::storage_dissolve_planning_node_api,
            planning::storage_restore_dissolved_planning_node_api,
            planning::storage_set_planning_node_issue_api,
            labels::storage_list_labels_api,
            labels::storage_create_label_api,
            labels::storage_update_label_api,
            labels::storage_archive_label_api,
            labels::storage_restore_archived_label_api,
            labels::storage_trash_label_api,
            labels::storage_restore_trashed_label_api,
            storage::storage_load_preferences_api,
            storage::storage_save_preferences_api,
            portability::storage_create_backup_api,
            portability::storage_list_backups_api,
            portability::storage_restore_backup_api,
            portability::storage_export_portable_file_api,
            portability::storage_export_backup_file_api,
            portability::storage_begin_import_upload_api,
            portability::storage_append_import_chunk_api,
            portability::storage_discard_import_upload_api,
            portability::storage_inspect_import_upload_api,
            portability::storage_finish_import_upload_api,
            portability::storage_run_restore_catchup_api,
            media::storage_stage_image_api,
            media::storage_read_image_api,
            time::storage_list_time_api,
            time::storage_list_timer_recovery_notices_api,
            time::storage_resolve_timer_recovery_notice_api,
            time::storage_list_workspace_time_api,
            time::storage_list_workspace_time_page_api,
            time::storage_running_timer_api,
            time::storage_running_timers_api,
            time::storage_timer_state_version_api,
            time::storage_start_timer_api,
            time::storage_stop_timer_api,
            time::storage_add_time_api,
            time::storage_update_time_api,
            time::storage_delete_time_api,
            cycles::storage_list_cycles_api,
            cycles::storage_list_cycle_series_api,
            cycles::storage_set_cycle_series_enabled_api,
            cycles::storage_list_cycle_reconcile_progress_api,
            cycles::storage_configure_cycles_api,
            cycles::storage_update_cycle_schedule_api,
            cycles::storage_create_cycle_api,
            cycles::storage_update_cycle_api,
            cycles::storage_set_cycle_issue_api,
            cycles::storage_set_cycle_successor_api,
            cycles::storage_complete_cycle_api,
            search::storage_search_issues_api,
            search::storage_search_projects_api,
            search::storage_search_planning_nodes_api,
            search::storage_search_comments_api,
            search::storage_find_comment_api,
            search::storage_query_issue_id_page_api,
            labels::storage_list_owner_labels_api,
            views::storage_list_views_api,
            views::storage_create_view_api,
            views::storage_update_view_api,
            views::storage_delete_view_api,
            issues::storage_issue_counts_api,
            issues::storage_issue_descendant_count_api,
            issues::storage_list_issue_children_api,
            issues::storage_issue_statuses_api,
            issues::storage_list_issue_page_api,
            issues::storage_list_issue_records_api,
            workflow::storage_list_workflow_api,
            workflow::storage_save_workflow_api,
            issues::storage_list_issue_relations_api,
            issues::storage_create_issue_relation_api,
            issues::storage_delete_issue_relation_api,
            issues::storage_list_issue_activity_api,
            issues::storage_list_issue_status_visits_api,
            issues::storage_create_issue_comment_api,
            issues::storage_bulk_triage_api,
            issues::storage_update_issue_comment_api,
            issues::storage_remove_issue_comment_api,
            issues::storage_preview_issue_move_api,
            issues::storage_move_issue_project_api,
            issues::storage_create_issue_api,
            issues::storage_update_issue_api,
            issues::storage_reorder_issues_api,
            issues::storage_archive_issue_api,
            issues::storage_archive_issue_with_strategy_api,
            issues::storage_delete_archived_issue_api,
            issues::storage_restore_issue_api,
            issues::storage_restore_sub_issue_api
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| match event {
            tauri::RunEvent::WindowEvent {
                label,
                event: tauri::WindowEvent::CloseRequested { api, .. },
                ..
            } if label == "main" => {
                if !shutdown::close_allowed() {
                    api.prevent_close();
                    let _ = app.emit("app-close-requested", ());
                } else if !shutdown::force_close() {
                    if let Err(error) = flush_timers_before_exit(app) {
                        eprintln!("Cannot close safely: {error}");
                        api.prevent_close();
                        shutdown::reset_close();
                        let _ = app.emit("app-close-requested", ());
                    }
                }
            }
            tauri::RunEvent::ExitRequested { api, .. } => {
                if !shutdown::close_allowed() {
                    api.prevent_exit();
                    let _ = app.emit("app-close-requested", ());
                }
            }
            tauri::RunEvent::Exit => {
                if !shutdown::force_close() {
                    if let Err(error) = flush_timers_before_exit(app) {
                        eprintln!("Cannot finish timer shutdown: {error}");
                    }
                }
            }
            _ => {}
        });
}
