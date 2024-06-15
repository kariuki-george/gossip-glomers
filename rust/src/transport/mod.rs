use crate::events::Message;

#[derive(Debug, Default)]
pub struct Transport;

impl Transport {
    pub fn handleinput(&self, input: String) -> Result<Message, ()> {
        let message = match serde_json::from_str::<Message>(&input) {
            Ok(message) => message,
            Err(err) => {
                handle_deserialization_error(err, input);
                return Err(());
            }
        };
        Ok(message)
    }
    pub fn handleoutput(&self, message: Message) {
        match serde_json::to_string(&message) {
            Ok(output) => println!("{output}"),
            Err(err) => handle_serialization_error(err, message),
        };
    }
}

fn handle_serialization_error(error: serde_json::Error, message: Message) {
    log::error!(
        "failed to serialize message: \n Message: {:?} \n err: {:?} ",
        message,
        error
    )
}

fn handle_deserialization_error(error: serde_json::Error, input: String) {
    log::warn!(
        "failed to deserialize input: \n Input {} \n err {:?}",
        input,
        error
    )
}
