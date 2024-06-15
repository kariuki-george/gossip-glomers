use gossip_glommers::{node, transport::Transport};
use std::io::BufRead;

#[tokio::main]
async fn main() {
    let input_lines: std::io::StdinLock<'_> = std::io::stdin().lock();

    let transport = Transport {};

    let mut node = node::Node::new().await;

    for line in input_lines.lines() {
        let line = line.unwrap();
        // Try to deserialize the input into a struct

        let message = match transport.handleinput(line) {
            Ok(message) => message,
            Err(_) => continue,
        };

        // Run the message event
        let reply_message = node.runner(message.clone()).await;

        // Check if to reply
        let mut reply_message = match reply_message {
            Some(message) => message,
            None => continue,
        };

        // Response sequence
        //        Set origins
        if reply_message.dest.is_empty() {
            reply_message.dest = message.src;
            reply_message.src = message.dest;
        }

        // Response
        transport.handleoutput(reply_message);
    }
}
