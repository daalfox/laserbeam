use std::io::{stdin, stdout, BufRead, StdoutLock, Write};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::from_str;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message<Payload> {
    src: String,
    dest: String,
    pub body: Body<Payload>,
}

impl<Payload> Message<Payload> {
    fn into_reply(self, msg_id: usize, reply_payload: Payload) -> Self {
        Self {
            src: self.dest,
            dest: self.src,
            body: self.body.into_reply(msg_id, reply_payload),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Body<Payload> {
    #[serde(rename = "msg_id")]
    id: Option<usize>,
    in_reply_to: Option<usize>,
    #[serde(flatten)]
    pub payload: Payload,
}
impl<Payload> Body<Payload> {
    fn into_reply(self, msg_id: usize, reply_payload: Payload) -> Self {
        Self {
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

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum InitOk {
    InitOk,
}

pub trait Node {
    type Payload;
    fn spawn<P>() -> anyhow::Result<()>
    where
        Self: Sized,
        Self::Payload: for<'a> Deserialize<'a>,
        Message<Self::Payload>: Serialize,
    {
        let mut stdin = stdin().lock();
        let mut stdout = stdout().lock();

        let mut init_msg = String::new();
        stdin.read_line(&mut init_msg)?;

        let init_msg: Message<Init> = from_str(&init_msg)?;

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

    fn from_init(init_msg: Message<Init>) -> Self;

    fn reply_init(&self, msg: Message<Init>, stdout: &mut StdoutLock) -> anyhow::Result<()> {
        serde_json::to_writer(
            &mut *stdout,
            &Message {
                src: msg.dest,
                dest: msg.src,
                body: Body {
                    id: Some(0),
                    in_reply_to: msg.body.id,
                    payload: InitOk::InitOk,
                },
            },
        )?;
        stdout
            .write_all(b"\n")
            .context("failed to write trailing new line")?;

        Ok(())
    }

    fn handle(&mut self, input: &Self::Payload) -> Option<(usize, <Self as Node>::Payload)>;

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
