pub enum Message {
    ApplyCollectionsFilter,
    CancelFilteringCollectionMode,
    FilterFromSelectingCollectionMode,
    SelectCollection(String),
    LoadMoreData,
}

impl Message {
    // Determines if this message should trigger loading
    pub fn should_trigger_loading(&self) -> bool {
        matches!(self, Message::SelectCollection(_) | Message::LoadMoreData)
    }

    // Returns an optional loading message
    pub fn loading_message(&self) -> Option<&str> {
        match self {
            Message::SelectCollection(_) => Some("Fetching Data..."),
            Message::LoadMoreData => Some("Loading More Data..."),
            _ => None,
        }
    }
}
