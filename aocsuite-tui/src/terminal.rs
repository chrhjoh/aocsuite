use std::io::{self, Stdout};

use crossterm::{
    cursor::Show,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub(crate) type TuiTerminal = Terminal<CrosstermBackend<Stdout>>;

pub(crate) struct TerminalSession {
    terminal: TuiTerminal,
    active: bool,
}

impl TerminalSession {
    pub(crate) fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        let terminal = match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => terminal,
            Err(error) => {
                let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
                let _ = disable_raw_mode();
                return Err(error);
            }
        };
        Ok(Self {
            terminal,
            active: true,
        })
    }

    pub(crate) fn terminal_mut(&mut self) -> &mut TuiTerminal {
        &mut self.terminal
    }

    pub(crate) fn suspend(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        self.terminal.show_cursor()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        disable_raw_mode()?;
        self.active = false;
        Ok(())
    }

    pub(crate) fn resume(&mut self) -> io::Result<()> {
        if self.active {
            return Ok(());
        }
        enable_raw_mode()?;
        if let Err(error) = execute!(io::stdout(), EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        self.terminal.clear()?;
        self.terminal.hide_cursor()?;
        self.active = true;
        Ok(())
    }

    fn restore(&mut self) {
        let _ = self.terminal.show_cursor();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, Show);
        let _ = disable_raw_mode();
        self.active = false;
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.restore();
    }
}
