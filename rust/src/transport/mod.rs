use crate::{events::Message, node::Node};

pub fn handleinput(input: String, node: &mut Node) -> Message {
    serde_json::from_str::<Message>(&input)
        .unwrap_or_else(|err| node.handle_deserialization_error(err))
}
pub fn handleoutput(message: Message, node: &mut Node) {
    let output = serde_json::to_string(&message)
        .unwrap_or_else(move |err| node.handle_serialization_error(err, message));
    println!("{output}");
}
