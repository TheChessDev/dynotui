use std::io;

use app::App;

mod app;
mod components;
mod util;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::new()?.run(&mut terminal);

    ratatui::restore();

    app_result
}
