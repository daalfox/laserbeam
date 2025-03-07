use laserbeam::{Init, Message, Node};
use serde::{Deserialize, Serialize};

struct EchoNode {
    last_sent_message: usize,
}

impl Node for EchoNode {
    type Payload = Payload;

    fn from_init(_init_msg: Message<Init>) -> Self {
        Self {
            last_sent_message: 0,
        }
    }

    fn handle(&mut self, input: &Self::Payload) -> Option<(usize, Self::Payload)> {
        match input {
            Payload::Echo { echo } => {
                self.last_sent_message += 1;
                Some((
                    self.last_sent_message,
                    Payload::EchoOk {
                        echo: echo.to_string(),
                    },
                ))
            }
            Payload::EchoOk { .. } => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

fn main() -> anyhow::Result<()> {
    EchoNode::spawn()
}
