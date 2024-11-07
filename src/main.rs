use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use data::{load_collections, FetchRequest, FetchResponse};
use tokio::{sync::mpsc, task};

use crate::app::App;

mod action;
mod app;
mod cli;
mod components;
mod config;
mod constants;
mod data;
mod errors;
mod logging;
mod tui;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    // Set up channels
    let (fetch_tx, mut fetch_rx) = mpsc::channel(10);
    let (response_tx, response_rx) = mpsc::channel(10);

    // Spawn the background task
    task::spawn(async move {
        while let Some(request) = fetch_rx.recv().await {
            match request {
                FetchRequest::FetchTables => {
                    let collections = load_collections().await;
                    let _ = response_tx
                        .send(FetchResponse::TablesFetched(collections))
                        .await;
                }
            }
        }
    });

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate, fetch_tx, response_rx)?;
    app.run().await?;
    Ok(())
}
