use std::io::{Write};


use crossterm::{cursor, ExecutableCommand, style, terminal};
use crossterm::cursor::MoveTo;
use crossterm::style::{Attribute, Color, PrintStyledContent, SetAttribute, SetBackgroundColor, SetForegroundColor, StyledContent};
use crossterm::terminal::{Clear, ClearType};
use log::{warn};


use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct CrosstermOutput<W: Write> {
    stdout: W,
    size: XY,
    front_buffer: BufferOutput,
    back_buffer: BufferOutput,
    current_buffer: bool,
}

impl<W: Write> CrosstermOutput<W> {
    pub fn new(mut stdout: W) -> Self {
        write!(stdout, "{}", cursor::Hide).unwrap();

        let size: XY = terminal::size().unwrap().into();

        let front_buffer = BufferOutput::new(size);
        let back_buffer = BufferOutput::new(size);

        CrosstermOutput {
            stdout,
            size,
            front_buffer,
            back_buffer,
            current_buffer: false,
        }
    }

    // returns new size or none if no update happened.
    pub fn update_size(&mut self) -> Option<XY> {
        let new_size: XY = terminal::size().unwrap().into();
        if new_size != self.size {
            self.size = new_size;

            self.back_buffer = BufferOutput::new(self.size);
            self.front_buffer = BufferOutput::new(self.size);

            Some(new_size)
        } else {
            None
        }
    }

    fn reset_cursor(&mut self) -> Result<(), std::io::Error> {
        self.stdout
            .execute(Clear(ClearType::All))?
            .execute(SetForegroundColor(Color::Reset))?
            .execute(SetBackgroundColor(Color::Reset))?
            .execute(SetAttribute(Attribute::Reset))?
            .execute(cursor::MoveTo(0, 0))?
            .execute(cursor::Show)?;
        Ok(())
    }

    pub fn end_frame(&mut self) -> Result<(), std::io::Error> {
        if log::log_enabled!(log::Level::Debug) {
            let size: XY = terminal::size().unwrap().into();
            debug_assert!(
                self.size == size,
                "output size different that crossterm size!"
            );
        }

        let buffer = if self.current_buffer == false {
            &self.front_buffer
        } else {
            &self.back_buffer
        };

        self.stdout
            .execute(Clear(ClearType::All))?
            .execute(SetForegroundColor(Color::Reset))?
            .execute(SetBackgroundColor(Color::Reset))?
            .execute(SetAttribute(Attribute::Reset))?;


        for y in 0..self.size.y {
            self.stdout.execute(cursor::MoveTo(0, y)); //TODO HANDLE ERR

            for x in 0..self.size.x {
                let pos = (x, y).into();

                let cell = &buffer[pos];
                match cell {
                    Cell::Begin { style, grapheme } => {
                        let bgcolor = Color::Rgb {
                            r: style.background.R,
                            g: style.background.G,
                            b: style.background.B,
                        };
                        let fgcolor = Color::Rgb {
                            r: style.foreground.R,
                            g: style.foreground.G,
                            b: style.foreground.B,
                        };

                        let mut cstyle = style::ContentStyle::new();
                        cstyle.background_color = Some(bgcolor);
                        cstyle.foreground_color = Some(fgcolor);

                        match style.effect {
                            Effect::Bold => {
                                cstyle.attributes.set(Attribute::Bold);
                            }
                            Effect::Italic => {
                                cstyle.attributes.set(Attribute::Italic);
                            }
                            Effect::Underline => {
                                cstyle.attributes.set(Attribute::Underlined);
                            }
                            _ => {}
                        };


                        self.stdout
                            .execute(MoveTo(pos.x, pos.y))?
                            .execute(PrintStyledContent(StyledContent::new(
                                cstyle,
                                grapheme,
                            )))?;
                    }
                    Cell::Continuation => {}
                }
            }
        }

        self.stdout.flush()
    }
}

impl<W: Write> Output for CrosstermOutput<W> {
    fn print_at(&mut self, pos: XY, style: TextStyle, text: &str) {
        let buffer = if self.current_buffer == false {
            &mut self.front_buffer
        } else {
            &mut self.back_buffer
        };

        // debug!("printing {} at {}", text, pos);

        buffer.print_at(pos, style, text)
    }

    fn clear(&mut self) {
        match self.stdout.execute(Clear(ClearType::All)) {
            Ok(_) => {}
            Err(err) => {
                warn!("failed to clear output, {}", err);
            }
        }


        self.current_buffer = !self.current_buffer;
        let buffer = if self.current_buffer == false {
            &mut self.front_buffer
        } else {
            &mut self.back_buffer
        };
        buffer.clear()
    }
}

impl<W: Write> SizedXY for CrosstermOutput<W> {
    fn size(&self) -> XY {
        let (x, y) = termion::terminal_size().unwrap();
        // debug!("termion size: {},{}",x,y);
        XY::new(x, y)
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
