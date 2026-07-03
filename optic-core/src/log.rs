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
