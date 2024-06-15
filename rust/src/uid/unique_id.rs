use tokio::sync::Mutex;

#[derive(Debug, Default)]
pub struct UID {
    message_counter: Mutex<u64>,
}
#[derive(Debug)]
pub enum UidError {
    GenerateError,
}

impl UID {
    pub fn new() -> UID {
        UID::default()
    }

    pub async fn generate_unique_id(&mut self, node_id: &str) -> Result<String, UidError> {
        // Generate a snowflake with the following parts
        // Timestamp in milliseconds.
        // The node identifier. A node can generate 1000 ids per second without any breaking uniqueness with other nodes
        // A node's sequence. Ensure uniqueness within a node when it generates more than 1 id within the same millisecond.

        let mut message_counter = self.message_counter.lock().await;

        let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => {
                log::error!("failed to generate timestamp");
                return Err(UidError::GenerateError);
            }
        };

        let snowflake = format!("{}-{}-{}", timestamp, node_id, message_counter);
        *message_counter += 1;
        Ok(snowflake)
    }
    pub async fn generate_int_unique_id(&mut self) -> Result<u64, UidError> {
        // Generates an integer id that doesn't clash with the above snowflake implementation

        let mut message_counter = self.message_counter.lock().await;
        let id = *message_counter;
        *message_counter += 1;
        Ok(id)
    }
}
