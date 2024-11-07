use std::collections::HashMap;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::{types::AttributeValue, Client, Error};
use serde_json::{Map, Value};

use crate::util::dynamodb_to_json;

#[derive(Debug)]
pub enum FetchRequest {
    Tables,
    TableData(String),
    NextBatchTableData(String, Option<HashMap<String, AttributeValue>>),
}

#[derive(Debug)]
pub enum FetchResponse {
    Tables(Vec<String>),
    TableData(Vec<String>, bool, Option<HashMap<String, AttributeValue>>),
    NextBatchTableData(Vec<String>, bool, Option<HashMap<String, AttributeValue>>),
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

pub async fn load_data(
    collection_name: &str,
    last_evaluated_key: Option<HashMap<String, AttributeValue>>,
) -> Result<(Vec<String>, bool, Option<HashMap<String, AttributeValue>>), Error> {
    let client = get_client().await;

    let mut request = client.scan().table_name(collection_name).limit(100);

    // Add `ExclusiveStartKey` if continuing from a previous batch
    if let Some(ref key) = last_evaluated_key {
        for (k, v) in key.iter() {
            request = request.exclusive_start_key(k.clone(), v.clone());
        }
    }

    let response = request.send().await?;

    // Collect records in JSON string format
    let records = if let Some(items) = response.items {
        items
            .into_iter()
            .map(|item| {
                let mut json_item = Map::new();
                for (k, v) in item {
                    json_item.insert(k, dynamodb_to_json(v));
                }
                Value::Object(json_item).to_string()
            })
            .collect()
    } else {
        Vec::new()
    };

    // Update pagination state
    let new_last_evaluated_key = response
        .last_evaluated_key
        .map(|key| key.into_iter().collect::<HashMap<String, AttributeValue>>());

    let has_more = new_last_evaluated_key.is_some();

    Ok((records, has_more, new_last_evaluated_key))
}
