use std::io::{Write};


use log::debug;
use termion::{clear, cursor, style, terminal_size};


use crate::io::buffer_output::BufferOutput;
use crate::io::cell::Cell;
use crate::io::output::Output;
use crate::io::style::{Effect, TextStyle};
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;

pub struct TermionOutput<W: Write> {
    stdout: W,
    size: XY,
    front_buffer: BufferOutput,
    back_buffer: BufferOutput,
    current_buffer: bool,
}

impl<W: Write> TermionOutput<W> {
    pub fn new(mut stdout: W) -> Self {
        write!(stdout, "{}", cursor::Hide).unwrap();

        let size: XY = terminal_size().unwrap().into();

        let front_buffer = BufferOutput::new(size);
        let back_buffer = BufferOutput::new(size);

        TermionOutput {
            stdout,
            size,
            front_buffer,
            back_buffer,
            current_buffer: false,
        }
    }

    // returns new size or none if no update happened.
    pub fn update_size(&mut self) -> Option<XY> {
        let new_size: XY = terminal_size().unwrap().into();
        if new_size != self.size {
            self.size = new_size;

            self.back_buffer = BufferOutput::new(self.size);
            self.front_buffer = BufferOutput::new(self.size);

            Some(new_size)
        } else {
            None
        }
    }

    pub fn end_frame(&mut self) {
        if log::log_enabled!(log::Level::Debug) {
            let size: XY = terminal_size().unwrap().into();
            debug_assert!(
                self.size == size,
                "output size different that termion size!"
            );
        }

        let buffer = if self.current_buffer == false {
            &self.front_buffer
        } else {
            &self.back_buffer
        };

        write!(
            self.stdout,
            "{}{}{}{}",
            clear::All,
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Black),
            termion::style::Reset
        )
        .unwrap();

        for y in 0..self.size.y {
            write!(self.stdout, "{}", cursor::Goto(1, (y + 1) as u16));

            for x in 0..self.size.x {
                let pos = (x, y).into();

                let cell = &buffer[pos];
                match cell {
                    Cell::Begin { style, grapheme } => {
                        let bgcolor = termion::color::Bg(termion::color::Rgb(
                            style.background.R,
                            style.background.G,
                            style.background.B,
                        ));
                        let fgcolor = termion::color::Fg(termion::color::Rgb(
                            style.foreground.R,
                            style.foreground.G,
                            style.foreground.B,
                        ));

                        match style.effect {
                            //TODO
                            Effect::None => {
                                write!(self.stdout, "{}", style::NoBold);
                                write!(self.stdout, "{}", style::NoItalic);
                                write!(self.stdout, "{}", style::NoUnderline);
                            }
                            Effect::Bold => {
                                write!(self.stdout, "{}", style::Bold);
                            }
                            Effect::Italic => {
                                write!(self.stdout, "{}", style::Italic);
                            }
                            Effect::Underline => {
                                write!(self.stdout, "{}", style::Underline);
                            }
                        };

                        write!(
                            self.stdout,
                            "{}{}{}{}",
                            cursor::Goto(pos.x + 1, pos.y + 1),
                            bgcolor,
                            fgcolor,
                            grapheme
                        );
                    }
                    Cell::Continuation => {}
                }
            }
        }

        self.stdout.flush().unwrap();
    }
}

impl<W: Write> Output for TermionOutput<W> {
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
        write!(self.stdout, "{}", clear::All);

        self.current_buffer = !self.current_buffer;
        let buffer = if self.current_buffer == false {
            &mut self.front_buffer
        } else {
            &mut self.back_buffer
        };

        buffer.clear()
    }
}

impl<W: Write> SizedXY for TermionOutput<W> {
    fn size(&self) -> XY {
        let (x, y) = termion::terminal_size().unwrap();
        debug!("termion size: {},{}",x,y);
        XY::new(x, y)
    }
}

impl<W: Write> Drop for TermionOutput<W> {
    fn drop(&mut self) {
        write!(
            self.stdout,
            "{}{}{}{}",
            clear::All,
            style::Reset,
            cursor::Goto(1, 1),
            cursor::Show
        )
        .unwrap();
        self.stdout.flush().unwrap();
    }
}
