use app::App;

mod app;
mod components;
mod message;
mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;
    let app_result = App::new().await?.run(&mut terminal).await;

    ratatui::restore();

    app_result
}
