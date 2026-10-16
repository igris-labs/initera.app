use std::ffi::{c_char, c_void};
use std::sync::OnceLock;

use objc2::runtime::{AnyClass, AnyObject, ProtocolObject, Sel};
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSApplicationDelegate};
use tauri::{AppHandle, Emitter};

use crate::shutdown;

static APP: OnceLock<AppHandle> = OnceLock::new();

#[link(name = "objc")]
extern "C" {
    fn class_addMethod(
        class: *const AnyClass,
        selector: Sel,
        implementation: *const c_void,
        types: *const c_char,
    ) -> i8;
}

extern "C" fn should_terminate(
    _delegate: *mut AnyObject,
    _selector: Sel,
    _application: *mut NSApplication,
) -> usize {
    if shutdown::close_allowed() {
        return 1; // NSTerminateNow
    }

    if let Some(app) = APP.get() {
        let _ = app.emit("app-close-requested", ());
    }
    0 // NSTerminateCancel: the renderer will retry or close after flushing.
}

pub(crate) fn install(app: &AppHandle) -> Result<(), String> {
    APP.set(app.clone())
        .map_err(|_| "Cannot register the macOS quit gate twice.".to_owned())?;

    let main_thread = MainThreadMarker::new().ok_or("macOS quit gate needs the main thread.")?;
    let application = NSApplication::sharedApplication(main_thread);
    let delegate = application
        .delegate()
        .ok_or("Cannot find the macOS application delegate.")?;
    let protocol: &ProtocolObject<dyn NSApplicationDelegate> = &delegate;
    let object: &AnyObject = protocol.as_ref();
    let class = object.class();

    // Tauri's macOS Quit path does not emit ExitRequested. Add this optional
    // delegate method before the app runs, while the delegate class is stable.
    // An existing implementation must be reviewed instead of overwritten.
    let added = unsafe {
        class_addMethod(
            class,
            sel!(applicationShouldTerminate:),
            should_terminate as *const c_void,
            b"Q@:@\0".as_ptr().cast(),
        )
    };
    if added == 0 {
        return Err("Cannot install the macOS quit gate on the application delegate.".into());
    }
    Ok(())
}
