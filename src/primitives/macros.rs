#[macro_export]
macro_rules! unpack_unit {
    ($ee: expr) => { $crate::unpack!($ee, ()) };
    ($ee: expr, $($msg:tt)+) => { $crate::unpack!($ee, (), Debug $($msg)+) };
}

#[macro_export]
macro_rules! unpack_unit_e {
    ($ee: expr, $($msg:tt)+) => { $crate::unpack!($ee, (), Error $($msg)+) };
}

#[macro_export]
macro_rules! unpack_quiet {
    ($ee: expr, $ret_val: expr) => {
        $crate::unpack!($ee, $ret_val)
    };
}

#[macro_export]
macro_rules! unpack_or {
    ($ee: expr, $ret_val: expr, $($msg:tt)+) => {
        $crate::unpack!($ee, $ret_val, Debug $($msg)+)
    };
}

#[macro_export]
macro_rules! unpack_or_e {
    ($ee: expr, $ret_val: expr, $($msg:tt)+) => {
        $crate::unpack!($ee, $ret_val, Error $($msg)+)
    };
}

#[macro_export]
macro_rules! unpack {
    ($ee: expr, $ret_val: expr $(, $level: ident $($msg:tt)+)?) => {
        match $ee {
            Some(item) => item,
            None => {
                $( log::log!(log::Level::$level, $($msg)+); )?
                return $ret_val;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, sync::Once};

    use log::Level;

    type TestLogs = Vec<(String, log::Level)>;

    thread_local! {
        static TEST_LOGS: RefCell<TestLogs> = RefCell::new(Vec::new());
    }
    static TEST_INIT: Once = Once::new();

    struct TestLogger {}

    impl TestLogger {
        pub fn setup() {
            TEST_INIT.call_once(|| {
                log::set_logger(&TestLogger {})
                    .expect("Logging already initialized by another test; run these tests in isolation for proper results");
                log::set_max_level(log::LevelFilter::Trace);
            });
            TEST_LOGS.with(|logs| logs.borrow_mut().clear());
        }

        pub fn drain_logs() -> TestLogs {
            TEST_LOGS.with(|logs| logs.borrow_mut().drain(..).collect())
        }
    }

    impl log::Log for TestLogger {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            if !self.enabled(record.metadata()) {
                return;
            }

            TEST_LOGS.with(|logs| {
                logs.borrow_mut().push((format!("{}", record.args()), record.level()));
            });
        }

        fn flush(&self) {}
    }

    #[test]
    fn test_interface_1() {
        let x = || -> Option<i32> {
            let x = unpack_quiet!(Some(3), None);
            Some(x + 1)
        };

        let none: Option<i32> = None;
        let y = || -> Option<i32> {
            let x = unpack_quiet!(none, None);
            Some(x + 1)
        };

        let z = || -> Option<i32> {
            let x = unpack_or!(none, None, "log this shit {:?}", none);
            Some(x + 1)
        };

        assert_eq!(x(), Some(4));
        assert_eq!(y(), None);
        assert_eq!(z(), None);
    }

    // Ignore the unpack* tests by default because they use a custom logger (TestLogger) to check
    // that the correct messages are being output, but the rest of the test suite uses
    // env_logger. Since logging can only be initialized once, if env_logger gets
    // initialized once, then these tests will produce incorrect asserts under the
    // assumption that the custom TestLogger is being used.

    #[test]
    #[ignore]
    fn test_unpack_unit() {
        TestLogger::setup();

        fn plain_return(expr: Option<usize>, inner: usize) {
            let some_val = unpack_unit!(expr);
            assert_eq!(some_val, inner);
        }
        assert_eq!(plain_return(Some(12), 12), ());
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(plain_return(None, 42), ());
        assert_eq!(TestLogger::drain_logs(), vec![]);

        fn plain_return_and_literal_msg(expr: Option<usize>, inner: usize) {
            let some_val = unpack_unit!(expr, "debug msg");
            assert_eq!(some_val, inner);
        }
        assert_eq!(plain_return_and_literal_msg(Some(12), 12), ());
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(plain_return_and_literal_msg(None, 42), ());
        assert_eq!(TestLogger::drain_logs(), vec![("debug msg".to_string(), Level::Debug)]);

        fn plain_return_and_formatted_msg(expr: Option<usize>, inner: usize) {
            let some_val = unpack_unit!(expr, "debug msg {} {}", 10, true);
            assert_eq!(some_val, inner);
        }
        assert_eq!(plain_return_and_formatted_msg(Some(12), 12), ());
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(plain_return_and_formatted_msg(None, 42), ());
        assert_eq!(TestLogger::drain_logs(), vec![("debug msg 10 true".to_string(), Level::Debug)]);
    }

    #[test]
    #[ignore]
    fn test_unpack_unit_e() {
        TestLogger::setup();

        fn plain_return_and_literal_msg(expr: Option<usize>, inner: usize) {
            let some_val = unpack_unit_e!(expr, "error msg");
            assert_eq!(some_val, inner);
        }
        assert_eq!(plain_return_and_literal_msg(Some(12), 12), ());
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(plain_return_and_literal_msg(None, 42), ());
        assert_eq!(TestLogger::drain_logs(), vec![("error msg".to_string(), Level::Error)]);

        fn plain_return_and_formatted_msg(expr: Option<usize>, inner: usize) {
            let some_val = unpack_unit_e!(expr, "error msg {} {}", 10, true);
            assert_eq!(some_val, inner);
        }
        assert_eq!(plain_return_and_formatted_msg(Some(12), 12), ());
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(plain_return_and_formatted_msg(None, 42), ());
        assert_eq!(TestLogger::drain_logs(), vec![("error msg 10 true".to_string(), Level::Error)]);
    }

    #[test]
    #[ignore]
    fn test_unpack_quiet() {
        TestLogger::setup();

        fn return_val(expr: Option<usize>, ret: usize) -> usize {
            let some_val = unpack_quiet!(expr, ret);
            some_val + 1
        }
        assert_eq!(return_val(Some(1), 0), 2);
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(return_val(None, 0), 0);
        assert_eq!(TestLogger::drain_logs(), vec![]);
    }

    #[test]
    #[ignore]
    fn test_unpack_or() {
        TestLogger::setup();

        fn return_val_and_literal_msg(expr: Option<usize>, ret: usize) -> usize {
            let some_val = unpack_or!(expr, ret, "debug msg");
            some_val + 1
        }
        assert_eq!(return_val_and_literal_msg(Some(1), 0), 2);
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(return_val_and_literal_msg(None, 0), 0);
        assert_eq!(TestLogger::drain_logs(), vec![("debug msg".to_string(), Level::Debug)]);

        fn return_val_and_formatted_msg(expr: Option<usize>, ret: usize) -> usize {
            let some_val = unpack_or!(expr, ret, "debug msg {} {}", 10, true);
            some_val + 1
        }
        assert_eq!(return_val_and_formatted_msg(Some(1), 0), 2);
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(return_val_and_formatted_msg(None, 0), 0);
        assert_eq!(TestLogger::drain_logs(), vec![("debug msg 10 true".to_string(), Level::Debug)]);
    }

    #[test]
    #[ignore]
    fn test_unpack_or_e() {
        TestLogger::setup();

        fn return_val_and_literal_msg(expr: Option<usize>, ret: usize) -> usize {
            let some_val = unpack_or_e!(expr, ret, "error msg");
            some_val + 1
        }
        assert_eq!(return_val_and_literal_msg(Some(1), 0), 2);
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(return_val_and_literal_msg(None, 0), 0);
        assert_eq!(TestLogger::drain_logs(), vec![("error msg".to_string(), Level::Error)]);

        fn return_val_and_formatted_msg(expr: Option<usize>, ret: usize) -> usize {
            let some_val = unpack_or_e!(expr, ret, "error msg {} {}", 10, true);
            some_val + 1
        }
        assert_eq!(return_val_and_formatted_msg(Some(1), 0), 2);
        assert_eq!(TestLogger::drain_logs(), vec![]);
        assert_eq!(return_val_and_formatted_msg(None, 0), 0);
        assert_eq!(TestLogger::drain_logs(), vec![("error msg 10 true".to_string(), Level::Error)]);
    }
}
