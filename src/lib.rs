use std::io::{stdin, stdout, BufRead, StdoutLock, Write};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message<P> {
    src: String,
    dest: String,
    pub body: Body<P>,
}

impl<P> Message<P> {
    pub fn into_reply<U>(self, msg_id: usize, reply_payload: U) -> Message<U> {
        Message {
            src: self.dest,
            dest: self.src,
            body: self.body.into_reply(msg_id, reply_payload),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Body<P> {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: P,
}
impl<P> Body<P> {
    fn into_reply<U>(self, msg_id: usize, reply_payload: U) -> Body<U> {
        Body {
            id: Some(msg_id),
            in_reply_to: self.id,
            payload: reply_payload,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node {
    type Payload;

    fn from_init(init_msg: Message<Init>) -> Self;

    fn handle(&mut self, input: &Self::Payload) -> Option<(usize, Self::Payload)>;

    fn spawn() -> anyhow::Result<()>
    where
        Self: Sized,
        Self::Payload: for<'a> Deserialize<'a>,
        Message<Self::Payload>: Serialize,
    {
        let mut stdin = stdin().lock();
        let mut stdout = stdout().lock();

        let mut init_msg = String::new();
        stdin.read_line(&mut init_msg)?;

        let init_msg: Message<Init> = serde_json::from_str(&init_msg)?;

        let inputs =
            serde_json::Deserializer::from_reader(stdin).into_iter::<Message<Self::Payload>>();

        let mut node = Self::from_init(init_msg.clone());

        node.reply_init(init_msg, &mut stdout)?;

        for input in inputs {
            let input = input.context("couldn't deserialize input from maelstrom")?;

            if let Some((msg_id, reply_payload)) = node.handle(&input.body.payload) {
                node.reply(input.into_reply(msg_id, reply_payload), &mut stdout)?;
            }
        }

        Ok(())
    }

    fn reply_init(&self, init_msg: Message<Init>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        serde_json::to_writer(
            &mut *stdout,
            &init_msg.into_reply(0, json!({ "type": "init_ok" })),
        )?;
        stdout
            .write_all(b"\n")
            .context("failed to write trailing new line")?;

        Ok(())
    }

    fn reply(
        &self,
        message: Message<Self::Payload>,
        stdout: &mut StdoutLock,
    ) -> Result<(), anyhow::Error>
    where
        Message<Self::Payload>: Serialize,
    {
        serde_json::to_writer(&mut *stdout, &message).context("couldn't serialize response")?;
        stdout
            .write_all(b"\n")
            .context("failed to write trailing new line")?;

        Ok(())
    }
}
