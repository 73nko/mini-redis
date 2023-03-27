use serde::Deserialize;
use serde_json::Error;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Command {
    Get,
    Set,
    Delete,
    Exists,
    Keys,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub commands: Vec<CommandRequest>,
}

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: Command,
    pub key: Option<String>,
    pub value: Option<String>,
    pub expiration: Option<u64>,
    pub pattern: Option<String>,
}

pub fn parse_request(request_data: &str) -> Result<Request, Error> {
    serde_json::from_str(request_data)
}
