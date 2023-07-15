use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use log::error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::color::Color;
use crate::primitives::rect::Rect;
use crate::primitives::xy::XY;
use crate::unpack_or;

pub fn get_next_filename(dir: &Path, prefix: &str, suffix: &str) -> Option<PathBuf> {
    return match fs::read_dir(&dir) {
        Err(e) => {
            error!("failed read_dir: {:?}", e);
            None
        }
        Ok(contents) => {
            let all_files = contents
                .map(|r| r.ok().map(|de| {
                    de.path()
                        .file_name()
                        .map(|c| c.to_string_lossy().to_string())
                }))
                .flatten()
                .flatten()
                .collect::<HashSet<String>>();

            let mut idx: usize = 0;
            let mut filename = format!("{}{}{}", prefix, idx, suffix);
            while all_files.contains(&filename) {
                idx += 1;
                filename = format!("{}{}{}", prefix, idx, suffix);
            }

            Some(dir.join(filename))
        }
    };
}

pub fn fill_output(color: Color, output: &mut dyn Output) {
    let style = TextStyle::new(
        Color::new(0, 0, 0),
        color,
        Effect::None,
    );

    let mut rect = output.visible_rect();
    // let mut rect = Rect::from_zero(output.size());

    // this test just protects substractions in the loop below.
    if rect.lower_right().x < 1 || rect.lower_right().y < 1 {
        error!("degenerated rect, skipping fill_output");
        return;
    }

    let parent_size = output.size();
    if !(rect.lower_right() <= parent_size) {
        error!("visible rect outside output size, that's definitely an error. Restoring by imposing artificial limit");
        rect.size = output.size() - rect.pos;
        debug_assert!(rect.lower_right() <= parent_size);
    }

    for x in rect.upper_left().x..rect.lower_right().x {
        for y in rect.upper_left().y..rect.lower_right().y {
            output.print_at(
                XY::new(x, y),
                style,
                " ",
            )
        }
    }
}

pub fn copy_first_n_columns(s: &str, n: usize, allow_shorter: bool) -> Option<String> {
    let mut res = String::new();
    let mut cols: usize = 0;
    for g in s.graphemes(true) {
        if cols < n {
            res += g;
            cols += g.width();
        } else {
            break;
        }
    }
    debug_assert!(res.width() <= n);

    if res.width() == n {
        return Some(res);
    } else {
        if allow_shorter {
            Some(res)
        } else {
            None
        }
    }
}

pub fn copy_last_n_columns(s: &str, n: usize, allow_shorter: bool) -> Option<String> {
    let mut graphemes: Vec<&str> = vec![];
    let mut cols: usize = 0;
    for g in s.graphemes(true).rev() {
        if cols < n {
            graphemes.push(g);
            cols += g.width();
        } else {
            break;
        }
    }
    let res = graphemes.into_iter().rev().fold(String::new(), |a, b| a + b);

    debug_assert!(res.width() <= n, "{} !<= {}", res.width(), n);

    if res.width() == n {
        return Some(res);
    } else {
        if allow_shorter {
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_first_n_columns() {
        let sentence = "Quel est votre film préféré?";

        assert_eq!(copy_first_n_columns(sentence, 5, false), Some("Quel ".to_string()));
        assert_eq!(copy_first_n_columns(sentence, 100, false), None);
        assert_eq!(copy_first_n_columns(sentence, 100, true), Some(sentence.to_string()));
        assert_eq!(copy_first_n_columns(sentence, 25, true), Some("Quel est votre film préfé".to_string()));
    }

    #[test]
    fn test_copy_last_n_columns() {
        let sentence = "Quel est votre film préféré?";

        assert_eq!(copy_last_n_columns(sentence, 5, false), Some("féré?".to_string()));
        assert_eq!(copy_last_n_columns(sentence, 100, false), None);
        assert_eq!(copy_last_n_columns(sentence, 100, true), Some(sentence.to_string()));
        assert_eq!(copy_last_n_columns(sentence, 9, true), Some(" préféré?".to_string()));
    }
}