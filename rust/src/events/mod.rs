use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub src: String,
    pub dest: String,
    pub body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    #[serde(flatten)]
    pub typ: Event,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    Init {
        #[serde(flatten)]
        init: InitEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    InitOk {
        #[serde(flatten)]
        event_response: EventResponse,
    },

    Echo {
        #[serde(flatten)]
        echo: EchoEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    EchoOk {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        echo: EchoEvent,
    },
    Error {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        error: ErrorEvent,
    },

    Topology {
        #[serde(flatten)]
        topology: TopologyEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    TopologyOk {
        #[serde(flatten)]
        event_response: EventResponse,
    },
    Broadcast {
        #[serde(flatten)]
        broadcast: BroadcastEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    BroadcastOk {
        #[serde(flatten)]
        event_response: EventResponse,
    },
    Read {
        #[serde(flatten)]
        read: ReadEvent,
    },
    ReadOk {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        read_ok: ReadOkEvent,
    },
    Unsupported {
        #[serde(flatten)]
        shared: SharedEvent,
    },
    Generate {
        #[serde(flatten)]
        shared: SharedEvent,
    },
    GenerateOk {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        generate_ok: GenerateOk,
    },
    Send {
        #[serde(flatten)]
        send: SendEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    SendOk {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        send_ok: SendOkEvent,
    },
    Poll {
        #[serde(flatten)]
        poll: PollEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    PollOk {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        poll_ok: PollOkEvent,
    },
    CommitOffsets {
        #[serde(flatten)]
        commit_offsets: CommitOffsetsEvent,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    CommitOffsetsOk {
        #[serde(flatten)]
        event_response: EventResponse,
    },
    ListCommittedOffsets {
        #[serde(flatten)]
        list_committed_offsets: ListCommittedOffsets,
        #[serde(flatten)]
        shared: SharedEvent,
    },
    ListCommittedOffsetsOk {
        #[serde(flatten)]
        event_response: EventResponse,
        #[serde(flatten)]
        list_committed_offsets_ok: ListCommittedOffsetsOk,
    },
}

// Shared
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedEvent {
    pub msg_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResponse {
    pub in_reply_to: u64,
}

// Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub code: u64,
    pub text: String,
}

// Generate Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOk {
    pub id: String,
}

// Init

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitEvent {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

// Topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyEvent {
    pub topology: HashMap<String, Vec<String>>,
}

// Echo

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoEvent {
    pub echo: String,
}

// Broadcast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastEvent {
    pub message: serde_json::Value,
}

// Read
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadEvent {
    pub msg_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadOkEvent {
    pub messages: Vec<serde_json::Value>,
}

// Log
// Send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEvent {
    pub key: String,
    pub msg: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendOkEvent {
    pub offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollEvent {
    pub offsets: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollOkEvent {
    pub msgs: HashMap<String, Vec<Vec<serde_json::Value>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitOffsetsEvent {
    pub offsets: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommittedOffsets {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommittedOffsetsOk {
    pub offsets: HashMap<String, u64>,
}
