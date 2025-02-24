#![allow(dead_code)]

use std::io::{stdin, stdout, StdoutLock, Write};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Debug, Deserialize, Serialize)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

struct Node {
    id: String,
    last_sent_msg_id: usize,
}

impl Node {
    fn new() -> Self {
        Self {
            id: "".to_string(),
            last_sent_msg_id: 0,
        }
    }

    fn handle(&mut self, input: Message, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        let id = self.last_sent_msg_id + 1;

        let mut reply = None;
        match input.body.payload {
            Payload::Init { node_id, .. } => {
                self.id = node_id;

                reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                }
                .into();
            }
            Payload::Echo { echo } => {
                reply = Message {
                    src: input.dest,
                    dest: input.src,
                    body: Body {
                        id: Some(id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                }
                .into();
            }
            Payload::InitOk => {}
            Payload::EchoOk { .. } => {}
        };

        if let Some(reply) = reply {
            serde_json::to_writer(&mut *stdout, &reply).context("couldn't serialize response")?;
            stdout
                .write_all(b"\n")
                .context("failed to write trailing new line")?;
        };

        self.last_sent_msg_id = id;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = stdin().lock();
    let mut stdout = stdout().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for input in inputs {
        let input = input.context("couldn't deserialize input from maelstrom")?;

        let mut node = Node::new();

        node.handle(input, &mut stdout)
            .context("couldn't handle request")?;
    }

    Ok(())
}
