use crate::{events::*, transport::handleoutput};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Node {
    pub message_counter: u64,
    pub node_id: String,
    pub node_ids: Vec<String>,
    pub topology: HashMap<String, Vec<String>>,
    pub messages: Vec<serde_json::Value>,
    pub topology_set: HashSet<String>, // Allows the node to have the whole topology without duplicates.
    pub id_counter: u64,
}

impl Node {
    pub fn reply(&mut self) {
        self.message_counter += 1
    }

    pub fn runner(&mut self, message: Message) -> Option<Message> {
        // Match the event type
        match message.clone().body.typ {
            Event::Init { init, shared } => self.handle_init(init, shared),
            Event::InitOk { .. } => None,
            Event::Echo { echo, shared } => self.handle_echo(echo, shared),
            Event::EchoOk { .. } => None,
            Event::Error { .. } => Some(message.clone()),
            Event::Unsupported { shared } => self.handle_unsupported_error(shared),
            Event::Topology { topology, shared } => self.handle_topology(topology, shared),
            Event::TopologyOk { .. } => None,
            Event::Broadcast { broadcast, shared } => {
                self.handle_broadcast(broadcast, shared, &message)
            }
            Event::BroadcastOk { .. } => None,
            Event::Read { read } => self.handle_read(read),
            Event::ReadOk {
                event_response,
                read_ok,
            } => self.handle_read_ok(event_response, read_ok),
            Event::Generate { shared } => self.handle_generate(shared),
            Event::GenerateOk { .. } => None,
        }
    }

    fn handle_read(&mut self, read: ReadEvent) -> Option<Message> {
        Some(Message {
            dest: String::new(),
            src: String::new(),
            body: Body {
                typ: Event::ReadOk {
                    event_response: EventResponse {
                        in_reply_to: read.msg_id,
                    },
                    read_ok: ReadOkEvent {
                        messages: self.messages.clone(),
                    },
                },
            },
        })
    }

    fn handle_read_ok(
        &mut self,

        event_response: EventResponse,
        data: ReadOkEvent,
    ) -> Option<Message> {
        let _sender = event_response.in_reply_to;
        let _messages = data.messages;

        None
    }

    fn in_reply_to(&mut self, message: Message) -> u64 {
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
            Event::Topology {
                topology: _,
                shared,
            } => shared.msg_id,
            Event::TopologyOk { .. } => 0,
            Event::Broadcast {
                broadcast: _,
                shared,
            } => shared.msg_id,
            Event::BroadcastOk { .. } => 0,
            Event::Read { .. } => 0,
            Event::ReadOk {
                event_response,
                read_ok: _,
            } => event_response.in_reply_to,
            Event::Generate { shared } => shared.msg_id,
            Event::GenerateOk { .. } => 0,
        }
    }

    fn handle_error(&self, shared: SharedEvent) -> Option<Message> {
        let err = Message {
            dest: String::new(),
            src: String::new(),
            body: Body {
                typ: Event::Error {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                    error: ErrorEvent {
                        code: 1003,
                        text: "failed to generate id".to_string(),
                    },
                },
            },
        };
        Some(err)
    }

    fn handle_generate(&mut self, shared: SharedEvent) -> Option<Message> {
        // Generate a snowflake with the following parts
        // Timestamp in milliseconds.
        // The node identifier. A node can generate 1000 ids per second without any breaking uniqueness with other nodes
        // A node's sequence. Ensure uniqueness within a node when it generates more than 1 id within the same millisecond.

        let timestamp = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(_) => return self.handle_error(shared),
        };

        let snowflake = format!("{}-{}-{}", timestamp, self.node_id, self.id_counter);
        let message = Message {
            src: String::new(),
            dest: String::new(),
            body: Body {
                typ: Event::GenerateOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                    generate_ok: GenerateOk { id: snowflake },
                },
            },
        };

        self.id_counter += 1;

        Some(message)
    }

    fn handle_topology(&mut self, data: TopologyEvent, shared: SharedEvent) -> Option<Message> {
        self.topology.clone_from(&data.topology);

        for node in data.topology {
            self.topology_set.insert(node.0);

            for node in node.1 {
                self.topology_set.insert(node);
            }
        }

        Some(Message {
            dest: String::new(),
            src: String::new(),
            body: Body {
                typ: Event::TopologyOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                },
            },
        })
    }

    fn handle_broadcast(
        &mut self,
        data: BroadcastEvent,
        shared: SharedEvent,
        message: &Message,
    ) -> Option<Message> {
        self.messages.push(data.message.clone());

        // To prevent the nodes from broadcasting the same message infinitely,
        // A node will not broadcast back to the sender.
        // The topology will guarantee that the message will be send to a node only once.

        // Considerations: Consider only storing the nodes to which the node will broadcast to.

        for (node_id, node_ids) in self.topology.clone() {
            if node_id != self.node_id {
                continue;
            }

            for node_id in node_ids {
                // Don't sent the message back to source
                if node_id == message.src {
                    continue;
                }
                self.broadcast(
                    node_id,
                    BroadcastEvent {
                        message: data.message.clone(),
                    },
                )
            }
            // No need to loop over the rest
            break;
        }

        Some(Message {
            dest: String::new(),
            src: String::new(),
            body: Body {
                typ: Event::BroadcastOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                },
            },
        })
    }

    fn broadcast(&mut self, dest: String, data: BroadcastEvent) {
        let message = Message {
            src: self.node_id.clone(),
            dest: dest.to_string(),
            body: Body {
                typ: Event::Broadcast {
                    broadcast: data,
                    shared: SharedEvent {
                        msg_id: self.message_counter,
                    },
                },
            },
        };
        self.increment_message_counter();

        self.worker(message);
    }

    fn worker(&mut self, message: Message) {
        handleoutput(message, self);
    }
    fn increment_message_counter(&mut self) {
        self.message_counter += 1
    }

    fn handle_unsupported_error(&mut self, shared: SharedEvent) -> Option<Message> {
        Some(Message {
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
        })
    }

    pub fn handle_serialization_error(
        &mut self,
        error: serde_json::Error,
        message: Message,
    ) -> String {
        format!(
            "{:?}",
            Message {
                body: Body {
                    typ: Event::Error {
                        event_response: EventResponse {
                            in_reply_to: self.in_reply_to(message.clone())
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

    pub fn handle_deserialization_error(&mut self, error: serde_json::Error) -> Message {
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

    fn handle_init(&mut self, data: InitEvent, shared: SharedEvent) -> Option<Message> {
        self.node_id = data.node_id;
        self.node_ids = data.node_ids;

        Some(Message {
            dest: String::new(),
            src: String::new(),
            body: Body {
                typ: Event::InitOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                },
            },
        })
    }

    fn handle_echo(&mut self, data: EchoEvent, shared: SharedEvent) -> Option<Message> {
        Some(Message {
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
        })
    }
}
