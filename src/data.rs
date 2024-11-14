use std::collections::HashMap;

use aws_config::{meta::region::RegionProviderChain, BehaviorVersion};
use aws_sdk_dynamodb::{
    types::{AttributeValue, KeyType},
    Client, Error,
};
use serde_json::{Map, Value};

use crate::util::dynamodb_to_json;

#[derive(Debug)]
pub enum FetchRequest {
    Tables,
    TableData(String),
    NextBatchTableData(String, Option<HashMap<String, AttributeValue>>),
    GetApproximateItemCount(String),
    DescribeTable(String),
    QueryTableByPk(String, String, String),
    QueryTableByPkSk(String, String, String, String, String),
}

#[derive(Debug)]
pub enum FetchResponse {
    Tables(Vec<String>),
    TableData(Vec<String>, bool, Option<HashMap<String, AttributeValue>>),
    NextBatchTableData(Vec<String>, bool, Option<HashMap<String, AttributeValue>>),
    ApproximateTableDataCount(i64),
    TableDescription((Option<String>, Option<String>)),
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

    if let Some(ref key) = last_evaluated_key {
        for (k, v) in key.iter() {
            request = request.exclusive_start_key(k.clone(), v.clone());
        }
    }

    let response = request.send().await?;

    let records = if let Some(items) = response.items {
        hashmap_to_json(items)
    } else {
        Vec::new()
    };

    let new_last_evaluated_key = response
        .last_evaluated_key
        .map(|key| key.into_iter().collect::<HashMap<String, AttributeValue>>());

    let has_more = new_last_evaluated_key.is_some();

    Ok((records, has_more, new_last_evaluated_key))
}

pub async fn get_approximate_item_count(table_name: &str) -> Result<i64, Error> {
    let client = get_client().await;
    let response = client
        .describe_table()
        .table_name(table_name)
        .send()
        .await?;
    if let Some(table) = response.table {
        Ok(table.item_count.unwrap_or(0))
    } else {
        Ok(0)
    }
}

pub async fn describe_table_key_schema(
    table_name: &str,
) -> Result<(Option<String>, Option<String>), Error> {
    let client = get_client().await;

    let table_info = client
        .describe_table()
        .table_name(table_name)
        .send()
        .await?;

    let table = table_info.table();

    if table.is_none() {
        return Ok((None, None));
    }

    let key_schema = table.unwrap().key_schema();

    let mut partition_key = None;
    let mut sort_key = None;

    for key_element in key_schema {
        match key_element.key_type() {
            KeyType::Hash => partition_key = Some(key_element.attribute_name().to_string()),
            KeyType::Range => sort_key = Some(key_element.attribute_name().to_string()),
            _ => (),
        }
    }

    Ok((partition_key, sort_key))
}

pub async fn query_by_partition_key(
    table_name: &str,
    partition_key_name: &str,
    partition_key_value: &str,
) -> Result<Vec<String>, Error> {
    let client = get_client().await;

    let response = client
        .query()
        .table_name(table_name)
        .key_condition_expression("#pk = :pkval")
        .expression_attribute_names("#pk", partition_key_name)
        .expression_attribute_values(":pkval", AttributeValue::S(partition_key_value.to_string()))
        .send()
        .await?;

    let records = if let Some(items) = response.items {
        hashmap_to_json(items)
    } else {
        Vec::new()
    };

    Ok(records)
}

pub async fn query_by_partition_and_sort_key(
    table_name: &str,
    partition_key_name: &str,
    partition_key_value: &str,
    sort_key_name: &str,
    sort_key_value: &str,
) -> Result<Vec<String>, Error> {
    let client = get_client().await;

    let response = client
        .query()
        .table_name(table_name)
        .key_condition_expression("#pk = :pkval AND #sk = :skval")
        .expression_attribute_names("#pk", partition_key_name)
        .expression_attribute_names("#sk", sort_key_name)
        .expression_attribute_values(":pkval", AttributeValue::S(partition_key_value.to_string()))
        .expression_attribute_values(":skval", AttributeValue::S(sort_key_value.to_string()))
        .send()
        .await?;

    let records = if let Some(items) = response.items {
        hashmap_to_json(items)
    } else {
        Vec::new()
    };

    Ok(records)
}

fn hashmap_to_json(items: Vec<HashMap<String, AttributeValue>>) -> Vec<String> {
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
}
