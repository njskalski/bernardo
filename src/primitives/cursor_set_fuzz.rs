use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};

use crate::primitives::cursor::Selection;

impl<'a> Arbitrary<'a> for Selection {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let b: u16 = u.arbitrary()?;
        let len: u16 = u.arbitrary()?;
        Ok(Selection {
            b: b as usize,
            e: b as usize + len as usize,
        })
    }
}
