use crate::{log_fatal, log_info, log_warn};
use std::process;

pub const SUCCESS: i32 = 0;
pub const ERROR: i32 = 1;

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

pub fn end_success() {
    log_info!("process ended successfully! (code {SUCCESS})");
    process::exit(SUCCESS)
}

pub fn end_error() {
    log_fatal!("process ended due to fatal error! (code {ERROR})");
    process::exit(ERROR)
}
