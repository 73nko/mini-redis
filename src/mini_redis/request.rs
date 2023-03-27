use serde::Deserialize;
use serde_json::Error;

#[derive(Debug, Deserialize, PartialEq)]
pub enum Command {
    #[serde(alias = "GET")]
    Get,
    #[serde(alias = "SET")]
    Set,
    #[serde(alias = "DEL")]
    Delete,
    #[serde(alias = "EXISTS")]
    Exists,
    #[serde(alias = "KEYS")]
    Keys,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub command: Command,
    pub key: Option<String>,
    pub value: Option<String>,
    pub expiration: Option<u64>,
    pub pattern: Option<String>,
}

pub fn parse_request(request_data: &str) -> Result<Request, Error> {
    serde_json::from_str(request_data)
}
