use anyhow::{bail, Result};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct LineQueryCodec {
    next_index: usize,
}

impl LineQueryCodec {
    pub fn new() -> Self {
        Self { next_index: 0 }
    }
}

impl Decoder for LineQueryCodec {
    type Item = Request;

    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match src.iter().skip(self.next_index).position(|v| *v == b'\n') {
            Some(i) => {
                let newline_index = self.next_index + i + 1;
                let line = src.split_to(newline_index);
                let line = &line[..line.len() - 1];
                let line = String::from_utf8_lossy(line);

                self.next_index = 0;

                Request::parse(line.as_ref()).map(Some)
            }
            None => {
                self.next_index = src.len();
                Ok(None)
            }
        }
    }
}

impl Encoder<Response> for LineQueryCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode_to(dst);
        dst.reserve(1);
        dst.extend(b"\n");
        Ok(())
    }
}

pub enum Request {
    Get { key: String },
    Set { key: String, value: String },
}

impl Request {
    fn parse(src: &str) -> Result<Self> {
        // GET abc
        // SET abc 123
        let components = src.split(' ').collect::<Vec<_>>();

        match components[0] {
            "GET" => {
                if components.len() != 2 {
                    bail!("GET expects 1 argument separated by empty space")
                } else {
                    Ok(Request::Get {
                        key: components[1].to_owned(),
                    })
                }
            }
            "SET" => {
                if components.len() != 3 {
                    bail!("SET expects 2 arguments separated by empty space")
                } else {
                    Ok(Request::Set {
                        key: components[1].to_owned(),
                        value: components[2].to_owned(),
                    })
                }
            }
            c => bail!("Unknown command: {}", c),
        }
    }
}

pub enum Response {
    Get { key: String, value: Option<String> },
    Set { key: String },
}

impl Response {
    fn encode_to(self, dst: &mut BytesMut) {
        match self {
            Response::Set { key } => {
                let status = Status::Okay.encode();
                dst.reserve(status.len() + 1 + key.len());
                dst.extend(status.as_bytes());
                dst.extend(b" ");
                dst.extend(key.as_bytes());
            }
            Response::Get { key, value } => {
                let status = value
                    .as_ref()
                    .map(|_| Status::Okay)
                    .unwrap_or_else(|| Status::Fail)
                    .encode();

                dst.reserve(status.len() + 1 + key.len());
                dst.extend(status.as_bytes());
                dst.extend(b" ");
                dst.extend(key.as_bytes());

                value.map(|value| {
                    dst.reserve(1 + value.len());
                    dst.extend(b" ");
                    dst.extend(value.as_bytes());
                });
            }
        }
    }
}

enum Status {
    Okay,
    Fail,
}

impl Status {
    fn encode(&self) -> &'static str {
        match self {
            Status::Okay => "OKAY",
            Status::Fail => "FAIL",
        }
    }
}
