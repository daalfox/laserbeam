#![allow(dead_code)]

use std::{
    collections::HashMap,
    io::{stdin, stdout, StdoutLock, Write},
};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

impl Message {
    fn into_reply(self, issuer: &mut Node) -> Option<Self> {
        Some(Self {
            src: self.dest,
            dest: self.src,
            body: self.body.into_reply(issuer)?,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    payload: Payload,
}
impl Body {
    fn into_reply(self, issuer: &mut Node) -> Option<Self> {
        Some(Self {
            id: Some(issuer.increment_id()),
            in_reply_to: self.id,
            payload: self.payload.into_reply(issuer)?,
        })
    }
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
    Generate,
    GenerateOk {
        id: String,
    },
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
impl Payload {
    fn into_reply(self, issuer: &mut Node) -> Option<Self> {
        match self {
            Payload::Init { node_id, .. } => {
                issuer.id = node_id;
                Some(Payload::InitOk)
            }
            Payload::Echo { echo } => Some(Payload::EchoOk { echo }),
            Payload::Generate => Some(Payload::GenerateOk {
                id: issuer.gen_unique_id(),
            }),
            Payload::Broadcast { message } => {
                issuer.seen_broadcasts.push(message);
                Some(Payload::BroadcastOk)
            }
            Payload::Read => Some(Payload::ReadOk {
                messages: issuer.seen_broadcasts.clone(),
            }),
            Payload::Topology { .. } => Some(Payload::TopologyOk),
            Payload::InitOk => None,
            Payload::EchoOk { .. } => None,
            Payload::GenerateOk { .. } => None,
            Payload::BroadcastOk => None,
            Payload::ReadOk { .. } => None,
            Payload::TopologyOk => None,
        }
    }
}

struct Node {
    id: String,
    last_sent_msg_id: usize,
    seen_broadcasts: Vec<usize>,
}

impl Node {
    fn new() -> Self {
        Self {
            id: "".to_string(),
            last_sent_msg_id: 0,
            seen_broadcasts: Vec::new(),
        }
    }

    /// increments node's `last_sent_msg_id` field and returns it
    fn increment_id(&mut self) -> usize {
        self.last_sent_msg_id += 1;
        self.last_sent_msg_id
    }

    fn gen_unique_id(&self) -> String {
        eprintln!("generating unique id for node {}", self.id);
        let gen_id = format!("{}-{}", self.id, self.last_sent_msg_id + 1);
        gen_id
    }

    fn handle(&mut self, input: Message, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        if let Some(reply) = input.into_reply(self) {
            serde_json::to_writer(&mut *stdout, &reply).context("couldn't serialize response")?;
            stdout
                .write_all(b"\n")
                .context("failed to write trailing new line")?;
        };

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = stdin().lock();
    let mut stdout = stdout().lock();

    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    let mut node = Node::new();
    for input in inputs {
        let input = input.context("couldn't deserialize input from maelstrom")?;

        node.handle(input, &mut stdout)
            .context("couldn't handle request")?;
    }

    Ok(())
}
