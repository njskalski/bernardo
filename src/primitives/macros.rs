// TODO call Fizyk and golf this code

#[macro_export]
macro_rules! unpack_unit {
    ($ee: expr) => {
        match $ee {
            Some(item) => item,
            None => {
                return;
            }
        }
    };
    ($ee: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Debug, $msg, $($arg)*);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! unpack_unit_e {
    ($ee: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Error, $msg, $($arg)*);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! unpack_quiet {
    ($ee: expr, $ret_val: expr) => {
        match $ee {
            Some(item) => item,
            None => {
                return $ret_val;
            }
        }
    };
}

#[macro_export]
macro_rules! unpack_or {
    ($ee: expr, $ret_val: expr, $msg:literal) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Debug, $msg);
                return $ret_val;
            }
        }
    };
    ($ee: expr, $ret_val: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Debug, $msg, $($arg)*);
                return $ret_val;
            }
        }
    };
}

#[macro_export]
macro_rules! unpack_or_e {
    ($ee: expr, $ret_val: expr, $msg:literal) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Error, $msg);
                return $ret_val;
            }
        }
    };
    ($ee: expr, $ret_val: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Error, $msg, $($arg)*);
                return $ret_val;
            }
        }
    };
    ($ee: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Error, $msg, $($arg)*);
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! unpack_or_w {
    ($ee: expr, $ret_val: expr, $msg:literal) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Warn, $msg);
                return $ret_val;
            }
        }
    };
    ($ee: expr, $ret_val: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Warn, $msg, $($arg)*);
                return $ret_val;
            }
        }
    };
    ($ee: expr, $msg:literal, $($arg:tt)*) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Warn, $msg, $($arg)*);
                return;
            }
        }
    };
}

#[cfg(test)]
mod tests {
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

        let q = || {
            let _x = unpack_unit!(none, "log this shit {:?}", none);
        };

        assert_eq!(x(), Some(4));
        assert_eq!(y(), None);
        assert_eq!(z(), None);
        assert_eq!(q(), ());
    }
}
