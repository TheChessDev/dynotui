use aws_sdk_dynamodb::types::AttributeValue;
use serde_json::{json, Map, Value};

pub fn dynamodb_to_json(attr: AttributeValue) -> Value {
    match attr {
        AttributeValue::S(s) => json!(s),
        AttributeValue::N(n) => json!(n.parse::<f64>().unwrap_or(0.0)), // Convert string to number
        AttributeValue::Bool(b) => json!(b),
        AttributeValue::M(map) => {
            // Convert each entry in the map recursively to a serde_json `Map`
            let mut json_map = Map::new();
            for (k, v) in map {
                json_map.insert(k, dynamodb_to_json(v));
            }
            Value::Object(json_map)
        }
        AttributeValue::L(list) => {
            // Convert each item in the list recursively
            let json_list: Vec<Value> = list.into_iter().map(dynamodb_to_json).collect();
            json!(json_list)
        }
        AttributeValue::Null(_) => Value::Null,
        _ => Value::Null, // Handle unsupported types by returning `null`
    }
}
