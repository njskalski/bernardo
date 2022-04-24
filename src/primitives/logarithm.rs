use num::Unsigned;

#[inline]
pub fn ceil_log10_of_usize(what: usize) -> u16 {
    (what as f64).log10().ceil() as u16
}

#[cfg(test)]
mod tests {
    use crate::primitives::color::{BLACK, WHITE};

    use super::*;

    #[test]
    fn ceil_log10_of_usize_test() {
        assert_eq!(ceil_log10_of_usize(0), 1);
        assert_eq!(ceil_log10_of_usize(1), 1);
        assert_eq!(ceil_log10_of_usize(9), 1);
        assert_eq!(ceil_log10_of_usize(10), 2);
        assert_eq!(ceil_log10_of_usize(71), 2);
        assert_eq!(ceil_log10_of_usize(99), 2);
        assert_eq!(ceil_log10_of_usize(100), 3);
        assert_eq!(ceil_log10_of_usize(101), 3);
    }
}