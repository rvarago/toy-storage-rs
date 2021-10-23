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

impl Response {
    pub(super) fn status(&self) -> Status {
        match self {
            Response::Get { key: _, value } => {
                if value.is_some() {
                    Status::Okay
                } else {
                    Status::Fail
                }
            }
            Response::Set { key: _ } => Status::Okay,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum Status {
    Okay,
    Fail,
}
