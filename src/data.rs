use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::Client;

#[derive(Debug)]
pub enum FetchRequest {
    FetchTables,
}

#[derive(Debug)]
pub enum FetchResponse {
    TablesFetched(Vec<String>),
}

pub async fn get_client() -> Client {
    let region = "us-east-1";
    let region_provider = RegionProviderChain::default_provider().or_else(region);
    let config = aws_config::defaults(BehaviorVersion::v2024_03_28())
        .region(region_provider)
        .load()
        .await;

    Client::new(&config)
}

pub async fn load_collections() -> Vec<String> {
    let client = get_client().await;

    let mut last_evaluated_table_name = None;

    let mut collections = Vec::new();

    loop {
        let request = client
            .list_tables()
            .set_exclusive_start_table_name(last_evaluated_table_name.clone());

        match request.send().await {
            Ok(output) => {
                let table_names = output.table_names();

                for name in table_names {
                    collections.push(name.clone());
                }

                last_evaluated_table_name =
                    output.last_evaluated_table_name().map(|s| s.to_string());

                if last_evaluated_table_name.is_none() {
                    break;
                }
            }
            Err(_) => {
                collections.push("Error loading collections.".to_string());
                break;
            }
        }
    }

    collections
}
