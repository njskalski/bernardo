use std::fmt::{Debug, Formatter};
use std::io::Write;

use crossterm::style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{BeginSynchronizedUpdate, Clear, ClearType, EndSynchronizedUpdate};
use crossterm::{cursor, style, terminal, ExecutableCommand, QueueableCommand};
use log::{debug, warn};
use unicode_width::UnicodeWidthStr;

use crate::io::buffer_output::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::output::{FinalOutput, Output};
use crate::io::style::{Effect, TextStyle};
use crate::primitives::rect::Rect;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct CrosstermOutput<W: Write> {
    stdout: W,
    size: XY,
    buffer: BufferOutput,
}

impl<W: Write> CrosstermOutput<W> {
    pub fn new(mut stdout: W) -> Self {
        stdout.execute(cursor::Hide).unwrap();

        let size: XY = terminal::size().unwrap().into();

        let buffer = BufferOutput::new(size);

        CrosstermOutput { stdout, size, buffer }
    }

    // returns new size or none if no update happened.
    pub fn update_size(&mut self) -> Option<XY> {
        let new_size: XY = terminal::size().unwrap().into();
        if new_size != self.size {
            self.size = new_size;

            self.buffer = BufferOutput::new(self.size);

            Some(new_size)
        } else {
            None
        }
    }

    fn reset_cursor(&mut self) -> Result<(), std::io::Error> {
        self.stdout
            .queue(Clear(ClearType::All))?
            .queue(SetForegroundColor(Color::Reset))?
            .queue(SetBackgroundColor(Color::Reset))?
            .queue(SetAttribute(Attribute::Reset))?
            .queue(cursor::MoveTo(0, 0))?
            .queue(cursor::Show)?
            .flush()
    }

    pub fn get_back_buffer(&self) -> &BufferOutput {
        &self.buffer
    }
}

impl<W: Write> SizedXY for CrosstermOutput<W> {
    fn size(&self) -> XY {
        self.size
    }
}

impl<W: Write> Output for CrosstermOutput<W> {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        // debug!("printing {} at {}", text, pos);
        self.buffer.print_at(pos, style, text)
    }

    fn clear(&mut self) -> Result<(), std::io::Error> {
        self.buffer.clear()
    }

    fn visible_rect(&self) -> Rect {
        let res = Rect::from_zero(self.size());

        debug_assert!(res.lower_right() <= self.size);

        res
    }

    #[cfg(test)]
    fn emit_metadata(&mut self, _meta: crate::io::output::Metadata) {
        debug_assert!(false, "you should not be emmiting metadata to an actual output");
    }
}

impl<W: Write> FinalOutput for CrosstermOutput<W> {
    fn get_front_buffer(&self) -> &BufferOutput {
        &self.buffer
    }

    fn end_frame(&mut self) -> Result<(), std::io::Error> {
        if log::log_enabled!(log::Level::Debug) {
            let size: XY = terminal::size().unwrap().into();
            debug_assert!(self.size == size, "output size different that crossterm size!");
        }

        self.stdout.queue(BeginSynchronizedUpdate)?;
        // self.stdout.queue(Clear(ClearType::All))?; // for some reason, this creates tearing.
        // self.stdout.queue(SetForegroundColor(Color::Reset))?;
        // self.stdout.queue(SetBackgroundColor(Color::Reset))?;
        // self.stdout.queue(SetAttribute(Attribute::Reset))?;

        let mut last_style: Option<TextStyle> = None;
        let mut curr_pos: XY = XY::ZERO;

        self.stdout.queue(cursor::MoveTo(0, 0))?;

        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let pos = XY::new(x, y);

                let cell = &self.buffer[pos];
                // let old_cell = &other_buffer[pos];

                if pos != curr_pos {
                    self.stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
                    debug!("moving curr_pos: {} -> {}", pos, curr_pos);
                    curr_pos = pos;
                }

                if true {
                    match cell {
                        Cell::Begin { style, grapheme } => {
                            if last_style != Some(*style) {
                                let bgcolor = Color::Rgb {
                                    r: style.background.r,
                                    g: style.background.g,
                                    b: style.background.b,
                                };
                                let fgcolor = Color::Rgb {
                                    r: style.foreground.r,
                                    g: style.foreground.g,
                                    b: style.foreground.b,
                                };

                                let mut attributes: style::Attributes = style::Attributes::default();
                                match style.effect {
                                    Effect::Bold => {
                                        attributes.set(Attribute::Bold);
                                    }
                                    Effect::Italic => {
                                        attributes.set(Attribute::Italic);
                                    }
                                    Effect::Underline => {
                                        attributes.set(Attribute::Underlined);
                                    }
                                    _ => {}
                                };

                                self.stdout.queue(SetBackgroundColor(Color::Reset))?;
                                self.stdout.queue(SetBackgroundColor(bgcolor))?;

                                self.stdout.queue(SetForegroundColor(Color::Reset))?;
                                self.stdout.queue(SetForegroundColor(fgcolor))?;

                                // TODO setting attributes breaks things colors.
                                // self.stdout.queue(SetAttribute(Attribute::Reset))?;
                                // self.stdout.queue(SetAttributes(attributes))?;

                                last_style = Some(*style);
                            }

                            self.stdout.queue(Print(grapheme))?;

                            curr_pos.x += grapheme.width() as u16;
                        }
                        Cell::Continuation => {}
                    } // match
                } // cell != old_cell
            } // x

            curr_pos.y += 1;
            curr_pos.x = 0;
        }

        self.stdout.queue(EndSynchronizedUpdate)?;
        self.stdout.flush()
    }
}

impl<W: Write> Drop for CrosstermOutput<W> {
    fn drop(&mut self) {
        match self.reset_cursor() {
            Ok(_) => {}
            Err(err) => {
                warn!("error while dropping output, {}", err);
            }
        }

        match self.stdout.flush() {
            Ok(_) => {}
            Err(err) => {
                warn!("error while dropping output, {}", err);
            }
        }
    }
}

impl<W: Write> Debug for CrosstermOutput<W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "!RawCrosstermOutput!")
    }
}
