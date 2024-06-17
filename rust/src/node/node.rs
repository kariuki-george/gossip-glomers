use serde_json::Value;

use crate::{
    broadcast::broadcast::{BroadCastMessage, Broadcast},
    db::db::DB,
    events::*,
    log::log::KLog,
    uid::unique_id::UID,
};

#[derive(Debug)]
pub struct Node {
    pub node_id: String,
    uid: UID,
    broadcast: Broadcast,
    db: DB<String, Value>,
    klog: KLog,
}

impl Node {
    pub async fn new() -> Node {
        Node {
            broadcast: Broadcast::new().await,
            db: DB::new(),
            node_id: String::new(),
            uid: UID::new(),
            klog: KLog::new(),
        }
    }

    pub async fn runner(&mut self, message: Message) -> Option<Message> {
        // Match the event type
        match message.body.clone().typ {
            Event::Init { init, shared } => self.handle_init(init, shared),
            Event::InitOk { .. } => None,
            Event::Echo { echo, shared } => self.handle_echo(echo, shared),
            Event::EchoOk { .. } => None,
            Event::Error { .. } => Some(message.clone()),
            Event::Unsupported { shared } => self.handle_unsupported_error(shared),
            Event::Topology { topology, shared } => self.handle_topology(topology, shared).await,
            Event::TopologyOk { .. } => None,
            Event::Broadcast { broadcast, shared } => {
                self.handle_broadcast(broadcast, shared, &message).await
            }
            Event::BroadcastOk { event_response } => self.handle_broadcast_ok(event_response).await,
            Event::Read { read } => self.handle_read(read),
            Event::ReadOk {
                event_response,
                read_ok,
            } => self.handle_read_ok(event_response, read_ok),
            Event::Generate { shared } => self.handle_generate(shared).await,
            Event::GenerateOk { .. } => None,

            Event::Send { send, shared } => self.handle_send(shared, send),
            Event::SendOk { .. } => None,
            Event::Poll { poll, shared } => self.handle_poll(poll, shared),
            Event::PollOk { .. } => None,
            Event::CommitOffsets {
                commit_offsets,
                shared,
            } => self.handle_commit_offsets(commit_offsets, shared),
            Event::CommitOffsetsOk { .. } => None,
            Event::ListCommittedOffsets {
                list_committed_offsets,
                shared,
            } => self.handle_list_committed_offsets(list_committed_offsets, shared),
            Event::ListCommittedOffsetsOk { .. } => None,
        }
    }

    fn handle_read(&mut self, read: ReadEvent) -> Option<Message> {
        let mut messages = self.db.get_messages_as_value();
        messages.sort_by(|a, b| {
            a.as_u64()
                .unwrap()
                .partial_cmp(&b.as_u64().unwrap())
                .unwrap()
        });

        Some(Message {
            dest: String::new(),
            src: String::new(),
            body: Body {
                typ: Event::ReadOk {
                    event_response: EventResponse {
                        in_reply_to: read.msg_id,
                    },
                    read_ok: ReadOkEvent { messages },
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

    // fn in_reply_to(&mut self, message: Message) -> u64 {
    //     match message.body.typ {
    //         Event::Init { init: _, shared } => shared.msg_id,
    //         Event::InitOk { .. } => 0,
    //         Event::Echo { echo: _, shared } => shared.msg_id,
    //         Event::EchoOk { .. } => 0,
    //         Event::Error {
    //             event_response,
    //             error: _,
    //         } => event_response.in_reply_to,
    //         Event::Unsupported { shared } => shared.msg_id,
    //         Event::Topology {
    //             topology: _,
    //             shared,
    //         } => shared.msg_id,
    //         Event::TopologyOk { .. } => 0,
    //         Event::Broadcast {
    //             broadcast: _,
    //             shared,
    //         } => shared.msg_id,
    //         Event::BroadcastOk { .. } => 0,
    //         Event::Read { .. } => 0,
    //         Event::ReadOk {
    //             event_response,
    //             read_ok: _,
    //         } => event_response.in_reply_to,
    //         Event::Generate { shared } => shared.msg_id,
    //         Event::GenerateOk { .. } => 0,
    //     }
    // }

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

    async fn handle_generate(&mut self, shared: SharedEvent) -> Option<Message> {
        let snowflake = match self.uid.generate_unique_id(&self.node_id).await {
            Ok(id) => id,
            Err(_) => return self.handle_error(shared),
        };
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

        Some(message)
    }

    async fn handle_topology(
        &mut self,
        data: TopologyEvent,
        shared: SharedEvent,
    ) -> Option<Message> {
        for (node_id, nodes) in data.topology {
            if node_id != self.node_id {
                continue;
            }

            self.broadcast.set_topology(nodes).await;
            break;
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

    async fn handle_broadcast(
        &mut self,
        data: BroadcastEvent,
        shared: SharedEvent,
        message: &Message,
    ) -> Option<Message> {
        // Internal broadcasts will broadcast message as object of
        // message and message_id
        let mut payload: BroadCastMessage = BroadCastMessage::default();

        if data.message.is_object() {
            // Check if the data is already available

            let payload_value: BroadCastMessage = match serde_json::from_value(data.message.clone())
            {
                Ok(payload) => payload,
                Err(err) => {
                    eprintln!(
                        "failed to cast broadcast object into BroadCastMessage. \n ERR: {:?}",
                        err
                    );
                    return None;
                }
            };

            payload.data = payload_value.data;
            payload.dist_message_id = payload_value.dist_message_id;

            let value = self.db.get_message(&payload.dist_message_id);

            if value.is_some() {
                // No need to continue
                return None;
            }
            self.db
                .add_message(payload.dist_message_id.clone(), payload.data.to_owned());
        } else {
            let id = match self.uid.generate_unique_id(self.node_id.as_str()).await {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("failed to generate unique id: \n err: {:?}", err);
                    return None;
                }
            };

            payload.data = data.message.clone();
            payload.dist_message_id.clone_from(&id);
            self.db.add_message(id, data.message);
        };

        self.broadcast
            .handle_broadcast(&self.node_id, &message.src, &mut self.uid, payload)
            .await;

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

    async fn handle_broadcast_ok(&mut self, data: EventResponse) -> Option<Message> {
        self.broadcast.handle_broadcast_ok(data.in_reply_to).await;
        None
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

    fn handle_init(&mut self, data: InitEvent, shared: SharedEvent) -> Option<Message> {
        self.node_id = data.node_id;

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

    // Log

    fn handle_send(&mut self, shared: SharedEvent, send: SendEvent) -> Option<Message> {
        let offset = self.klog.handle_append(send.key, send.msg);

        let message = Message {
            src: String::new(),
            dest: String::new(),
            body: Body {
                typ: Event::SendOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                    send_ok: SendOkEvent { offset },
                },
            },
        };

        Some(message)
    }
    fn handle_poll(&mut self, data: PollEvent, shared: SharedEvent) -> Option<Message> {
        let messages = self.klog.handle_poll(data.offsets);

        let message = Message {
            src: String::new(),
            dest: String::new(),
            body: Body {
                typ: Event::PollOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                    poll_ok: PollOkEvent { msgs: messages },
                },
            },
        };

        Some(message)
    }
    fn handle_commit_offsets(
        &mut self,
        data: CommitOffsetsEvent,
        shared: SharedEvent,
    ) -> Option<Message> {
        self.klog.handle_commit_offsets(data.offsets);

        let message = Message {
            src: String::new(),
            dest: String::new(),
            body: Body {
                typ: Event::CommitOffsetsOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                },
            },
        };

        Some(message)
    }

    fn handle_list_committed_offsets(
        &mut self,
        data: ListCommittedOffsets,
        shared: SharedEvent,
    ) -> Option<Message> {
        let offsets = self.klog.handle_list_committed_offsets(data.keys);

        let message = Message {
            src: String::new(),
            dest: String::new(),
            body: Body {
                typ: Event::ListCommittedOffsetsOk {
                    event_response: EventResponse {
                        in_reply_to: shared.msg_id,
                    },
                    list_committed_offsets_ok: ListCommittedOffsetsOk { offsets },
                },
            },
        };

        Some(message)
    }
}
