use std::collections::HashMap;

use serde_json::Value;

#[derive(Debug, Default)]
pub struct KLog {
    logs: HashMap<String, Log>,
}

impl KLog {
    pub fn new() -> KLog {
        KLog::default()
    }
    pub fn handle_append(&mut self, key: String, message: Value) -> u64 {
        // Check if the log exists
        if !self.logs.contains_key(&key) {
            // Create log
            self.create_log(&key);
        }

        // The log should exist. So the unwrap is just to make the compiler happy
        let log = self.logs.get_mut(&key).unwrap();
        log.append(message)
    }

    fn create_log(&mut self, key: &str) {
        self.logs.insert(key.to_owned(), Log::new());
    }

    pub fn handle_poll(
        &mut self,
        offsets: HashMap<String, u64>,
    ) -> HashMap<String, Vec<Vec<Value>>> {
        let mut output = HashMap::new();

        for (log_key, offset) in offsets {
            let log = self.get_log(log_key.clone());

            output.insert(log_key, log.poll(offset));
        }

        output
    }

    fn get_log(&mut self, log_key: String) -> &Log {
        if !self.logs.contains_key(&log_key) {
            self.create_log(&log_key);
        }
        self.logs.get(&log_key).unwrap()
    }
    fn get_log_mut(&mut self, log_key: String) -> &mut Log {
        if !self.logs.contains_key(&log_key) {
            self.create_log(&log_key);
        }
        self.logs.get_mut(&log_key).unwrap()
    }

    pub fn handle_commit_offsets(&mut self, offsets: HashMap<String, u64>) {
        for (log_key, offset) in offsets {
            let log = self.get_log_mut(log_key.clone());
            log.committ_offsets(offset);
        }
    }

    pub fn handle_list_committed_offsets(&mut self, logs: Vec<String>) -> HashMap<String, u64> {
        let mut hash = HashMap::with_capacity(logs.len());
        for log_key in logs {
            let log = self.get_log(log_key.clone());

            let offset = log.list_committed_offset();

            hash.insert(log_key, offset);
        }
        hash
    }
}

#[derive(Debug, Default)]
struct Log {
    messages: Vec<Message>,
    offset: u64,
    committed_offset: u64,
}

impl Log {
    fn new() -> Log {
        Log::default()
    }

    fn append(&mut self, message: Value) -> u64 {
        // Create a new message
        let message = Message::new(self.offset, Some(message));
        self.offset += 1;
        self.messages.push(message);
        self.offset - 1
    }

    fn poll(&self, offset: u64) -> Vec<Vec<Value>> {
        // Return a max of 10 after the offset
        let max = 10;

        let mut messages: Vec<Vec<Value>> = Vec::with_capacity(max);

        //NOTE:  offset.try_into.unwrap will never unwrap in a 64 bit architecture
        for message in self
            .messages
            .iter()
            .skip(offset.try_into().unwrap())
            .take(max)
        {
            let m_message = vec![
                serde_json::Value::from(message.offset),
                message.message.to_owned().into(),
            ];

            messages.push(m_message);
        }
        messages
    }

    fn committ_offsets(&mut self, offset: u64) {
        self.committed_offset = offset;
    }

    fn list_committed_offset(&self) -> u64 {
        self.committed_offset
    }
}

#[derive(Debug)]
struct Message {
    offset: u64,
    message: Option<Value>,
}

impl Message {
    fn new(offset: u64, message: Option<Value>) -> Self {
        Message { offset, message }
    }
}
