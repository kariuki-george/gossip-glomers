use serde::Deserialize;
use serde_json::{json, Value};
use std::{borrow::BorrowMut, collections::HashSet, sync::Arc};
use tokio::sync::Mutex;

use crate::{
    db::db::DB,
    events::{Body, BroadcastEvent, Event, Message, SharedEvent},
    transport::Transport,
    uid::unique_id::UID,
};

#[derive(Debug)]
pub struct Broadcast {
    service: Arc<Mutex<Service>>,
}

#[derive(Debug, Default)]
struct Store {
    topology: HashSet<String>,
    db: DB<u64, BMessage>,
    transport: Transport,
}

#[derive(Debug, Default)]
struct Service {
    store: Store,
}

#[derive(Debug, Clone, Default)]
struct BMessage {
    data: Value,
    dest: String,
    src: String,
    // This id corresponds to the event_id. It is used at the transport level
    broadcast_event_message_id: u64,
    // A broadcast message will have the same id across all nodes
    dist_message_id: String,
}

#[derive(Deserialize, Default, Debug)]
pub struct BroadCastMessage {
    #[serde(rename(deserialize = "d_id"))]
    pub dist_message_id: String,
    #[serde(rename(deserialize = "d"))]
    pub data: Value,
}

impl Broadcast {
    pub async fn new() -> Broadcast {
        let service = Arc::new(Mutex::new(Service::new()));

        tokio::task::spawn(handle_broadworker(service.clone()));

        Broadcast { service }
    }

    pub async fn handle_broadcast(
        &mut self,
        parent_node_id: &str,
        src: &str,
        uid: &mut UID,
        payload: BroadCastMessage,
    ) {
        let mut service = self.service.lock().await;
        let service = service.borrow_mut();

        service
            .handle_broadcast(parent_node_id, src, uid, payload)
            .await;
    }
    pub async fn set_topology(&mut self, nodes: Vec<String>) {
        let mut service = self.service.lock().await;
        let service = service.borrow_mut();

        service.set_topology(nodes).await;
    }

    pub async fn handle_broadcast_ok(&mut self, message_id: u64) {
        let mut service = self.service.lock().await;
        let service = service.borrow_mut();

        service.handle_broadcast_ok(message_id);
    }
}

impl Service {
    fn new() -> Service {
        let store = Store::default();
        Service { store }
    }

    async fn set_topology(&mut self, nodes: Vec<String>) {
        for node_id in nodes {
            self.store.topology.insert(node_id);
        }
    }

    async fn handle_broadcast(
        &mut self,
        parent_node_id: &str,
        src: &str,
        uid: &mut UID,
        payload: BroadCastMessage,
    ) {
        // To prevent the nodes from broadcasting the same message infinitely,
        // A node will not broadcast back to the sender.
        // The topology will guarantee that the message will be send to a node only once.

        // However,
        // In the case of broadcast failures
        // A background worker will be used for this task. it will gossip unacked events every X ms after they are send.
        let topology = &self.store.topology;
        let db = &mut self.store.db;
        let mut messages = vec![];
        for node_id in topology {
            if node_id == src {
                continue;
            }
            let broadcast_message = BMessage {
                data: payload.data.clone(),
                dest: node_id.clone(),
                broadcast_event_message_id: match uid.generate_int_unique_id().await {
                    Ok(id) => id,
                    Err(err) => {
                        eprintln!("failed to generate unique int id: {:?}", err);
                        continue;
                    }
                },
                dist_message_id: payload.dist_message_id.clone(),
                src: parent_node_id.to_owned(),
            };
            // Store the message
            db.add_message(
                broadcast_message.broadcast_event_message_id,
                broadcast_message.clone(),
            );
            messages.push(broadcast_message);
        }

        // Choose to send message immediately or gossip the messages in the worker below
        // Get better latencies by choosing not to send the message immediately

        // for data in messages {
        //      self.broadcast(data)
        // }
    }

    fn broadcast(&mut self, data: BMessage) {
        // To cater for fault_tolerance, listen to the acknowledgements.
        // If an acknowledgement is not received within some period,
        // Resend the message.

        let m_string = json!({"d": data.data,"d_id":data.dist_message_id});

        let message = Message {
            src: data.src,
            dest: data.dest,
            body: Body {
                typ: Event::Broadcast {
                    broadcast: BroadcastEvent { message: m_string },
                    shared: SharedEvent {
                        msg_id: data.broadcast_event_message_id,
                    },
                },
            },
        };

        self.store.transport.handleoutput(message);
    }

    fn handle_broadcast_ok(&mut self, message_id: u64) {
        self.store.db.delete_message(&message_id);
    }
}

async fn handle_broadworker(service: Arc<Mutex<Service>>) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(40)).await;

        // Rebroadcast

        let mut service_lock = service.lock().await;
        let st = service_lock.borrow_mut();

        let messages = st.store.db.get_messages_as_value();

        for message in messages {
            st.broadcast(message);
        }
    }
}
