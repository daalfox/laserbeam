use laserbeam::{Init, Message, Node};
use serde::{Deserialize, Serialize};

struct EchoNode {
    id: String,
    last_sent_message: usize,
}

impl Node for EchoNode {
    type Payload = Payload;

    fn from_init(init_msg: Message<Init>) -> Self {
        Self {
            id: init_msg.body.payload.node_id,
            last_sent_message: 0,
        }
    }

    fn handle(&mut self, input: &Self::Payload) -> Option<(usize, Self::Payload)> {
        match input {
            Payload::Generate => {
                self.last_sent_message += 1;
                Some((
                    self.last_sent_message,
                    Payload::GenerateOk {
                        id: format!("{}-{}", self.id, self.last_sent_message),
                    },
                ))
            }
            Payload::GenerateOk { .. } => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Generate,
    GenerateOk { id: String },
}

fn main() -> anyhow::Result<()> {
    EchoNode::spawn::<Payload>()?;
    Ok(())
}
