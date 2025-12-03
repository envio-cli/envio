#[macro_export]
macro_rules! log_msg {
    ($level:expr, $color:ident, $msg:expr) => {{
        use colored::Colorize;
        let label = stringify!($level).$color();
        println!("{}: {}", label, $msg);
    }};
    ($level:expr, $color:ident, $fmt:expr, $($arg:tt)*) => {{
        use colored::Colorize;
        let label = stringify!($level).$color();
        println!("{}: {}", label, format!($fmt, $($arg)*));
    }};
}

#[macro_export]
macro_rules! success_msg {
    ($($args:tt)*) => { $crate::log_msg!(Success, green, $($args)*) };
}

#[macro_export]
macro_rules! warning_msg {
    ($($args:tt)*) => { $crate::log_msg!(Warning, yellow, $($args)*) };
}

#[macro_export]
macro_rules! error_msg {
    ($($args:tt)*) => { $crate::log_msg!(Error, red, $($args)*) };
}
