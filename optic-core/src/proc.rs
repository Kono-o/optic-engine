use crate::{log_fatal, log_info};
use std::process;

pub fn end_success() {
    let code = 0;
    log_info!("process ended successfully! (code {code})");
    process::exit(code)
}

pub fn end_error() {
    let code = 1;
    log_fatal!("process ended due to fatal error! (code {code})");
    process::exit(code)
}
