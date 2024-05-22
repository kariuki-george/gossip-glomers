use gossip_glommers::{
    node,
    transport::{self, handleoutput},
};
use std::io::BufRead;

fn main() {
    let input_lines: std::io::StdinLock<'_> = std::io::stdin().lock();

    let mut node = node::Node::default();

    for line in input_lines.lines() {
        let line = line.unwrap();
        // Try to deserialize the input into a struct

        let message = transport::handleinput(line, &mut node);

        // Run the message event
        let reply_message = node.runner(message.clone());

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
        handleoutput(reply_message, &mut node);
    }
}
