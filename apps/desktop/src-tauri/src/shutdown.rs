use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;

static CLOSE_ALLOWED: AtomicBool = AtomicBool::new(false);
static FORCE_CLOSE: AtomicBool = AtomicBool::new(false);

pub(crate) fn close_allowed() -> bool {
    CLOSE_ALLOWED.load(Ordering::SeqCst)
}

pub(crate) fn force_close() -> bool {
    FORCE_CLOSE.load(Ordering::SeqCst)
}

pub(crate) fn reset_close() {
    CLOSE_ALLOWED.store(false, Ordering::SeqCst);
}

#[api_contract_macros::command]
pub fn storage_finish_close(app: AppHandle, force: bool) -> Result<(), String> {
    if !force {
        crate::flush_timers_before_exit(&app)?;
    }
    FORCE_CLOSE.store(force, Ordering::SeqCst);
    CLOSE_ALLOWED.store(true, Ordering::SeqCst);
    Ok(())
}

#[api_contract_macros::command]
pub fn storage_prepare_update(app: AppHandle) -> Result<(), String> {
    crate::flush_timers_before_exit(&app)
}
