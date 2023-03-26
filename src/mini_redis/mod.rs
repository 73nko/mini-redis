mod commands;
mod request;

pub use commands::handle_connection;
pub use request::Request;
use tokio::sync::Mutex;

use std::{collections::HashMap, sync::Arc};

pub type KeyValueStore = Arc<Mutex<HashMap<String, String>>>;

#[cfg(test)]
mod tests;
