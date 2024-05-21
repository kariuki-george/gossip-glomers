use serde::{Deserialize, Serialize};
use std::io::BufRead;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Body {
    #[serde(flatten)]
    typ: Event,
}

#[derive(Debug)]
struct Node {
    _message_counter: u64,
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Event {
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
    Unsupported {
        #[serde(flatten)]
        shared: SharedEvent,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorEvent {
    code: u64,
    text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SharedEvent {
    msg_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InitEvent {
    node_id: String,
    node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventResponse {
    in_reply_to: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EchoEvent {
    echo: String,
}

fn main() {
    let input_lines = std::io::stdin().lock();
    let mut node = Node {
        _message_counter: 0,
        node_id: String::new(),
        node_ids: Vec::new(),
    };

    for line in input_lines.lines() {
        let line = line.unwrap();

        // Try to deserialize the input into a struct
        let message =
            serde_json::from_str::<Message>(&line).unwrap_or_else(handle_deserialization_error);

        // Match the event type
        let mut reply_message: Message = match message.clone().body.typ {
            Event::Init { init, shared } => handle_init(&mut node, init, shared),
            Event::InitOk { .. } => todo!(),
            Event::Echo { echo, shared } => handle_echo(echo, shared),
            Event::EchoOk { .. } => todo!(),
            Event::Error { .. } => message.clone(),
            Event::Unsupported { shared } => handle_unsupported_error(shared),
        };

        // Set sources
        reply_message.dest = message.src;
        reply_message.src = message.dest;

        // Serialize
        let output = serde_json::to_string(&reply_message)
            .unwrap_or_else(move |err| handle_serialization_error(err, reply_message));

        // Write into stdout
        println!("{output}");
    }
}

fn in_reply_to(message: Message) -> u64 {
    match message.body.typ {
        Event::Init { init: _, shared } => shared.msg_id,
        Event::InitOk { .. } => 0,
        Event::Echo { echo: _, shared } => shared.msg_id,
        Event::EchoOk { .. } => 0,
        Event::Error {
            event_response,
            error: _,
        } => event_response.in_reply_to,
        Event::Unsupported { shared } => shared.msg_id,
    }
}

fn handle_unsupported_error(shared: SharedEvent) -> Message {
    Message {
        dest: String::new(),
        src: String::new(),
        body: Body {
            typ: Event::Error {
                event_response: EventResponse {
                    in_reply_to: shared.msg_id,
                },
                error: ErrorEvent {
                    code: 10,
                    text: "Message not supported on this version.".to_string(),
                },
            },
        },
    }
}

fn handle_serialization_error(error: serde_json::Error, message: Message) -> String {
    format!(
        "{:?}",
        Message {
            body: Body {
                typ: Event::Error {
                    event_response: EventResponse {
                        in_reply_to: in_reply_to(message.clone())
                    },
                    error: ErrorEvent {
                        code: 1001,
                        text: format!("Cannot serialize the message: {:?}", error),
                    },
                },
            },
            ..message
        }
    )
}

fn handle_deserialization_error(error: serde_json::Error) -> Message {
    Message {
        dest: String::new(),
        src: String::new(),
        body: Body {
            typ: Event::Error {
                event_response: EventResponse { in_reply_to: 0 },
                error: ErrorEvent {
                    code: 1000,
                    text: format!("Cannot deserialize the message: {:?}", error),
                },
            },
        },
    }
}

fn handle_init(node: &mut Node, data: InitEvent, shared: SharedEvent) -> Message {
    node.node_id = data.node_id;
    node.node_ids = data.node_ids;

    Message {
        dest: String::new(),
        src: String::new(),
        body: Body {
            typ: Event::InitOk {
                event_response: EventResponse {
                    in_reply_to: shared.msg_id,
                },
            },
        },
    }
}

fn handle_echo(data: EchoEvent, shared: SharedEvent) -> Message {
    Message {
        src: String::new(),
        dest: String::new(),
        body: Body {
            typ: Event::EchoOk {
                event_response: EventResponse {
                    in_reply_to: shared.msg_id,
                },
                echo: EchoEvent { echo: data.echo },
            },
        },
    }
}
