#[macro_export]
macro_rules! unpack_or {
    ($ee: expr) => {
        match $ee {
            Some(item) => item,
            None => {
                return;
            }
        }
    };
    ($ee: expr, $msg:literal, $($arg:tt)+) => {
    match $ee {
        Some(item) => item,
        None => {
            log::log!(log::Level::Debug, $msg, $($arg)+);
            return;
        }
    }
   };
    ($ee: expr, $ret_val: expr) => {
        match $ee {
            Some(item) => item,
            None => {
                return $ret_val;
            }
        }
    };
    ($ee: expr, $ret_val: expr, $msg:literal, $($arg:tt)+) => {
        match $ee {
            Some(item) => item,
            None => {
                log::log!(log::Level::Debug, $msg, $($arg)+);
                return $ret_val;
            }
        }
    };

}

#[cfg(test)]
mod tests {
    #[test]
    fn test_interface_1() {
        let x = || -> Option<i32> {
            let x = unpack_or!(Some(3), None);
            Some(x + 1)
        };

        let none: Option<i32> = None;
        let y = || -> Option<i32> {
            let x = unpack_or!(none, None);
            Some(x + 1)
        };

        let z = || -> Option<i32> {
            let x = unpack_or!(none, None, "log this shit {:?}", none);
            Some(x + 1)
        };

        let q = || {
            let _x = unpack_or!(none, "log this shit {:?}", none);
        };

        assert_eq!(x(), Some(4));
        assert_eq!(y(), None);
        assert_eq!(z(), None);
        assert_eq!(q(), ());
    }
}