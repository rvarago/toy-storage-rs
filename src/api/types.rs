//! Request/Response for API interaction.

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Response {
    Get { key: String, value: Option<String> },
    Set { key: String },
}
