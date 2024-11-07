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
    SelectingTable,
    FilteringTables,
    SelectingRegion,
    SelectingData,
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