use crate::mini_redis::request::{Command, CommandRequest};
use crate::mini_redis::Request;

#[test]
fn test_deserialize_set_request() {
    let json_request = r#"{
        "command": "SET",
        "key": "mykey",
        "value": "myvalue"
    }"#;

    let request: CommandRequest = serde_json::from_str(json_request).unwrap();

    assert_eq!(request.command, Command::Set);
    assert_eq!(request.key.unwrap(), "mykey");
    assert_eq!(request.value.unwrap(), "myvalue");
}

#[test]
fn test_deserialize_get_request() {
    let json_request = r#"{
        "command": "GET",
        "key": "mykey"
    }"#;
    let request: CommandRequest = serde_json::from_str(json_request).unwrap();

    assert_eq!(request.command, Command::Get);
    assert_eq!(request.key.unwrap(), "mykey");
    assert!(request.value.is_none());
}
