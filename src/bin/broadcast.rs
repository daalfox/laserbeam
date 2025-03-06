use std::collections::HashMap;

use laserbeam::{Init, Message, Node};
use serde::{Deserialize, Serialize};

struct EchoNode {
    last_sent_message: usize,
    messages: Vec<usize>,
}

impl Node for EchoNode {
    type Payload = Payload;

    fn from_init(_init_msg: Message<Init>) -> Self {
        Self {
            last_sent_message: 0,
            messages: Vec::new(),
        }
    }

    fn handle(&mut self, input: &Self::Payload) -> Option<(usize, Self::Payload)> {
        if let Some(reply) = match input {
            Payload::Topology { .. } => Some(Payload::TopologyOk),
            Payload::Broadcast { message } => {
                self.messages.push(*message);
                Some(Payload::BroadcastOk)
            }
            Payload::Read => Some(Payload::ReadOk {
                messages: self.messages.clone(),
            }),

            Payload::TopologyOk | Payload::BroadcastOk | Payload::ReadOk { .. } => None,
        } {
            self.last_sent_message += 1;

            return Some((self.last_sent_message, reply));
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
}

fn main() -> anyhow::Result<()> {
    EchoNode::spawn::<Payload>()?;
    Ok(())
}
