use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};
use unicode_segmentation::UnicodeSegmentation;

use crate::primitives::cursor_set::{Cursor, CursorSet, Selection};
use crate::text::buffer_state::{BufferState, BufferType};

impl<'a> Arbitrary<'a> for BufferState {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let subtype = u.arbitrary::<BufferType>()?;

        let mut text = u.arbitrary::<String>()?;
        let mut bf = match subtype {
            BufferType::Full => {
                BufferState::full(None).with_text(text.clone())
            }
            BufferType::SingleLine => {
                text = text.replace("\n", "");
                BufferState::simplified_single_line().with_text(text.clone())
            }
        };

        let text_char_len = text.as_str().graphemes(true).count();
        let num_cursors_poses = if text_char_len > 2 {
            u.arbitrary::<usize>()? / text_char_len / 2
        } else {
            1
        };

        let mut poses: Vec<usize> = Vec::new();
        for _ in 0..num_cursors_poses {
            if text_char_len > 0 {
                poses.push(u.arbitrary::<usize>()? / text_char_len);
            } else {
                poses.push(0);
            }
        }
        poses.sort();
        let poses = poses;

        let head: bool = u.arbitrary()?;

        let simple = u.arbitrary::<bool>()?;
        let cursors: Vec<Cursor> = if simple {
            poses.into_iter().map(|p| {
                Cursor::new(p)
            }).collect()
        } else {
            let mut i: usize = 0;
            let mut cursors: Vec<Cursor> = Vec::new();
            if i > 1 {
                loop {
                    debug_assert!(poses[i] <= poses[i + 1]);
                    let smaller = poses[i];
                    let bigger = poses[i + 1];

                    if i + 1 >= poses.len() {
                        break;
                    }

                    cursors.push(if head {
                        if smaller != bigger {
                            Cursor::new(smaller).with_selection(Selection::new(smaller, bigger))
                        } else {
                            Cursor::new(smaller)
                        }
                    } else {
                        if smaller != bigger {
                            Cursor::new(bigger).with_selection(Selection::new(smaller, bigger))
                        } else {
                            Cursor::new(bigger)
                        }
                    });

                    i += 2;
                }
            } else {
                cursors.push(Cursor::new(0));
            }

            cursors
        };

        Ok(bf)
    }
}