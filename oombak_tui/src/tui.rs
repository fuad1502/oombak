use crate::error::OombakResult;

use crossterm::ExecutableCommand;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use std::io::Stdout;
use std::panic::PanicHookInfo;
use std::{io, panic, process};

pub fn init_terminal() -> OombakResult<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    io::stdout().execute(crossterm::terminal::EnterAlternateScreen)?;
    setup_panic_hook();
    let terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    Ok(terminal)
}

pub fn restore_terminal() -> OombakResult<()> {
    io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

fn setup_panic_hook() {
    let original_hook = panic::take_hook();
    let panic_handler = move |hook_info: &PanicHookInfo| {
        let _ = restore_terminal();
        original_hook(hook_info);
        process::exit(1);
    };
    panic::set_hook(Box::new(panic_handler));
}
