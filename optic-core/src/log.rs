/// Print colored output using an [`ANSI`](crate::ansi::ANSI) constant.
///
/// ```
/// use optic_core::*;
///
/// log_color!("processing: {}", RED, "item 1");
/// ```
#[macro_export]
macro_rules! log_color {
    ($fmt:expr, $color:expr) => {
        let fmt = format!($fmt);
        println!("{}{}{}", $color.prefix, fmt, $color.suffix);
    };
    ($fmt:expr, $color:expr, $($args:tt)*) => {
        let fmt = format!($fmt, $($args)*);
        println!("{}{}{}", $color.prefix, fmt, $color.suffix);
    };
}

/// Print a bold-blue `[EVENT]` message.
#[macro_export]
macro_rules! log_event {
    ($fmt:expr) => {
        let fmt = format!($fmt);
        println!("{}[EVENT] {}{}", $crate::ansi::BOLD_BLUE.prefix, fmt, $crate::ansi::BOLD_BLUE.suffix);
    };
    ($fmt:expr, $($args:tt)*) => {
        let fmt = format!($fmt, $($args)*);
        println!("{}[EVENT] {}{}", $crate::ansi::BOLD_BLUE.prefix, fmt, $crate::ansi::BOLD_BLUE.suffix);
    };
}

/// Print a bold-green `[INFO]` message.
#[macro_export]
macro_rules! log_info {
    ($fmt:expr) => {
        let fmt = format!($fmt);
        println!("{}[INFO] {}{}", $crate::ansi::BOLD_GREEN.prefix, fmt, $crate::ansi::BOLD_GREEN.suffix);
    };
    ($fmt:expr, $($args:tt)*) => {
        let fmt = format!($fmt, $($args)*);
        println!("{}[INFO] {}{}", $crate::ansi::BOLD_GREEN.prefix, fmt, $crate::ansi::BOLD_GREEN.suffix);
    };
}

/// Print a bold-yellow `[WARN]` message.
#[macro_export]
macro_rules! log_warn {
    ($fmt:expr) => {
        let fmt = format!($fmt);
        println!("{}[WARN] {}{}", $crate::ansi::BOLD_YELLOW.prefix, fmt, $crate::ansi::BOLD_YELLOW.suffix);
    };
    ($fmt:expr, $($args:tt)*) => {
        let fmt = format!($fmt, $($args)*);
        println!("{}[WARN] {}{}", $crate::ansi::BOLD_YELLOW.prefix, fmt, $crate::ansi::BOLD_YELLOW.suffix);
    };
}

/// Print a bold-red `[FATAL]` message, then abort the process.
#[macro_export]
macro_rules! log_fatal {
    ($fmt:expr) => {
        let fmt = format!($fmt);
        println!("{}[FATAL] {}{}", $crate::ansi::BOLD_RED.prefix, fmt, $crate::ansi::BOLD_RED.suffix);
    };
    ($fmt:expr, $($args:tt)*) => {
        let fmt = format!($fmt, $($args)*);
        println!("{}[FATAL] {}{}", $crate::ansi::BOLD_RED.prefix, fmt, $crate::ansi::BOLD_RED.suffix);
    };
}

/// Print a bold-red `[ERROR]` message.
#[macro_export]
macro_rules! log_error {
    ($fmt:expr) => {
        let fmt = format!($fmt);
        println!("{}[ERROR] {}{}", $crate::ansi::BOLD_RED.prefix, fmt, $crate::ansi::BOLD_RED.suffix);
    };
    ($fmt:expr, $($args:tt)*) => {
        let fmt = format!($fmt, $($args)*);
        println!("{}[ERROR] {}{}", $crate::ansi::BOLD_RED.prefix, fmt, $crate::ansi::BOLD_RED.suffix);
    };
}
