#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            tklog::info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            tklog::error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            tklog::warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            tklog::debug!($($arg)*);
        }
    };
}

// test
#[cfg(test)]
mod tests {
    #[test]
    fn test_logger() {
        log_info!("This is an info message");
        log_error!("This is an error message");
        log_warn!("This is a warn message");
        log_debug!("This is a debug message");
    }
}
