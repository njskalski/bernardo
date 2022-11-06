use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};

use crate::primitives::cursor_set::Selection;

impl<'a> Arbitrary<'a> for Selection {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let mut iter = u.arbitrary_iter::<u16>()?;

        let b = iter.next().unwrap().unwrap();
        let len = iter.next().unwrap().unwrap();
        Ok(Selection {
            b: b as usize,
            e: b as usize + len as usize,
        })
    }
}
