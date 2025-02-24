use crate::error::OombakTuiResult;

use crossterm::ExecutableCommand;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::io;
use std::io::Stdout;

pub fn init_terminal() -> OombakTuiResult<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    Ok(terminal)
}

pub fn restore_terminal() -> OombakTuiResult<()> {
    io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
