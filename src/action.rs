use std::collections::HashMap;

use aws_sdk_dynamodb::types::AttributeValue;
use serde::{Deserialize, Serialize};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    ClearScreen,
    Error(String),
    Help,

    SelectTableMode,
    SelectTablePrev,
    SelectTableNext,
    SelectTableScrollUp,
    SelectTableScrollDown,
    SelectTableFirst,
    SelectTableLast,
    SelectTable,
    TransmitSelectedTable(String),

    SelectDataMode,
    TransmitTableData(Vec<String>, bool),
    FetchTableData(String),

    FilteringTables,
    SelectingRegion,
    EnterInsertMode,
    ExitInsertMode,
    NewCharacter(char),
    DeleteCharacter,
    SubmitText,
    TransmitSubmittedText(String),
    FetchTables,
    TransmitTables(Vec<String>),
    StartLoading(String),
    StopLoading,
}
