use std::i64;

use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use data::{get_approximate_item_count, load_collections, load_data, FetchRequest, FetchResponse};
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
                FetchRequest::Tables => {
                    let collections = load_collections().await;
                    let _ = response_tx.send(FetchResponse::Tables(collections)).await;
                }
                FetchRequest::TableData(collection_name) => {
                    if let Ok(result) = load_data(&collection_name, None).await {
                        let (data, has_more, last_evaluated_key) = result;

                        let _ = response_tx
                            .send(FetchResponse::TableData(data, has_more, last_evaluated_key))
                            .await;
                    }
                }
                FetchRequest::NextBatchTableData(collection_name, last_evaluated_key) => {
                    if let Ok(result) = load_data(&collection_name, last_evaluated_key).await {
                        let (data, has_more, last_evaluated_key) = result;

                        let _ = response_tx
                            .send(FetchResponse::NextBatchTableData(
                                data,
                                has_more,
                                last_evaluated_key,
                            ))
                            .await;
                    }
                }
                FetchRequest::GetApproximateItemCount(collection_name) => {
                    if let Ok(result) = get_approximate_item_count(&collection_name).await {
                        let _ = response_tx
                            .send(FetchResponse::ApproximateTableDataCount(result))
                            .await;
                    } else {
                        let _ = response_tx
                            .send(FetchResponse::ApproximateTableDataCount(0))
                            .await;
                    }
                }
            }
        }
    });

    let args = Cli::parse();
    let mut app = App::new(args.tick_rate, args.frame_rate, fetch_tx, response_rx)?;
    app.run().await?;
    Ok(())
}
