use crate::io::crossterm_output::CrosstermOutput;
use crate::io::crossterm_input::CrosstermInput;
use crate::primitives::sized_xy::SizedXY;
use crate::primitives::xy::XY;
use crossterm::terminal;
use std::io::{stdout, Error, ErrorKind, Stdout};

pub type AppError = Box<dyn std::error::Error>;

/// App is an application wrapper that ensures a bernardo app is correctly initialized and deinitialized.
#[derive(Default)]
pub struct App {
    /// Applications that use alt-screen mode will be rendered in a full window view, similarly to
    /// how vim or nano work. By default, this option is disabled.
    use_alt_screen_mode: bool,
}

impl App {
    /// Initializing the bernardo tui app
    pub fn init() -> Self {
        Self::default()
    }

    /// Enable alt-screen mode
    pub fn with_alt_screen_mode(mut self) -> Self {
        self.use_alt_screen_mode = true;
        self
    }

    fn run_init_chain(&self) -> Result<(), AppError> {
        terminal::enable_raw_mode()?;

        if self.use_alt_screen_mode {
            crossterm::execute!(stdout(), terminal::EnterAlternateScreen)?;
        }

        Ok(())
    }

    fn run_teardown_chain(&self) -> Result<(), AppError> {
        if self.use_alt_screen_mode {
            crossterm::execute!(stdout(), terminal::LeaveAlternateScreen)?;
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }

    /// Runs the application with previously defined init and teardown chains. Takes a main
    /// application function as an argument.
    pub fn run_with<F>(self, f: F) -> Result<(), AppError>
    where
        F: FnOnce(CrosstermInput, CrosstermOutput<Stdout>),
    {
        self.run_init_chain()?;

        let input = CrosstermInput::new();
        let output = CrosstermOutput::new(stdout());

        if output.size() == XY::ZERO {
            return Err(Box::new(Error::new(
                ErrorKind::Other,
                "it seems like the screen has zero size",
            )));
        }

        f(input, output);

        self.run_teardown_chain()
    }
}
