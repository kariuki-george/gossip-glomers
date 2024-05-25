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
