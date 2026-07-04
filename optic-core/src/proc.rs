use crate::{log_fatal, log_info, log_warn};
use std::process;

/// Success exit code (0).
pub const SUCCESS: i32 = 0;
/// Error exit code (1).
pub const ERROR: i32 = 1;

/// Terminate the process with the given exit code.
///
/// Dispatches to [`end_success`] or [`end_error`] for known codes.
pub fn end(code: i32) {
    match code {
        SUCCESS => end_success(),
        ERROR => end_error(),
        _ => {
            log_warn!("ending with custom exit code: {code}");
            process::exit(code);
        }
    }
}

/// Print a success message and exit with code 0.
pub fn end_success() {
    log_info!("process ended successfully! (code {SUCCESS})");
    process::exit(SUCCESS)
}

/// Print a fatal error message and exit with code 1.
pub fn end_error() {
    log_fatal!("process ended due to fatal error! (code {ERROR})");
    process::exit(ERROR)
}
