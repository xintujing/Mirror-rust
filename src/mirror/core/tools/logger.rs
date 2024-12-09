#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        {
            use tklog::info;
            info!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        {
            use tklog::error;
            error!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        {
            use tklog::warn;
            warn!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        {
            use tklog::debug;
            debug!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        {
            use tklog::trace;
            trace!($($arg)*);
        }
    };
}

#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        {
            use tklog::fatal;
            fatal!($($arg)*);
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
        log_trace!("This is a trace message");
        log_fatal!("This is a fatal message");
    }
}
