//! Codec for the wire protocol through which requests/responses are exchanged.

use anyhow::{bail, Result};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Default, Debug)]
pub struct Codec {
    next_index: usize,
}

impl Decoder for Codec {
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

impl Encoder<Response> for Codec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.encode_to(dst);
        dst.reserve(1);
        dst.extend(b"\n");
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
                    bail!("GET expects 1 argument separated by whitespace")
                } else {
                    Ok(Request::Get {
                        key: components[1].to_owned(),
                    })
                }
            }
            "SET" => {
                if components.len() != 3 {
                    bail!("SET expects 2 arguments separated by whitespace")
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

#[derive(Debug)]
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
                    .unwrap_or(Status::Fail)
                    .encode();

                dst.reserve(status.len() + 1 + key.len());
                dst.extend(status.as_bytes());
                dst.extend(b" ");
                dst.extend(key.as_bytes());

                if let Some(value) = value {
                    dst.reserve(1 + value.len());
                    dst.extend(b" ");
                    dst.extend(value.as_bytes());
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn succeeds_to_encode_status() {
        let cases = vec![(Status::Okay, "OKAY"), (Status::Fail, "FAIL")];

        cases
            .into_iter()
            .for_each(|(status, expected_encoded_status)| {
                // Pre-condition.
                // Action.
                let encoded_status = status.encode();
                // Post-condition.
                assert_eq!(encoded_status, expected_encoded_status);
            });
    }

    proptest! {
        #[test]
        fn fails_to_decode_request_with_invalid_command(command in invalid_request_command()) {
            // Pre-condition.
            let mut message = BytesMut::from(format!("{}\n", command).as_str());
            let mut decoder = Codec::default();

            // Action.
            let request = decoder.decode(&mut message);

            // Post-condition.
            assert!(request.is_err());
            assert!(message.is_empty());
        }
    }

    #[test]
    fn fails_to_decodes_malformed_request() {
        let cases = vec![
            (b"GET\n".as_ref(), "get without key"),
            (b"SET\n".as_ref(), "set without key"),
            (b"SET key\n".as_ref(), "set without value"),
        ];

        cases.into_iter().for_each(|(message, reason)| {
            // Pre-condition.
            let mut decoder = Codec::default();
            let mut message = BytesMut::from(message);

            // Action.
            let request = decoder.decode(&mut message);

            // Post-condition.
            assert!(request.is_err(), "{}", reason);
            assert!(message.is_empty(), "{}", reason);
        });
    }

    #[test]
    fn succeeds_to_decode_wellformed_request() {
        let cases = vec![
            (
                b"GET key\n".as_ref(),
                Request::Get { key: "key".into() },
                "get key",
            ),
            (
                b"SET key value\n".as_ref(),
                Request::Set {
                    key: "key".into(),
                    value: "value".into(),
                },
                "set key to value",
            ),
        ];

        cases
            .into_iter()
            .for_each(|(message, expected_request, reason)| {
                // Pre-condition.
                let mut decoder = Codec::default();
                let mut message = BytesMut::from(message);

                // Action.
                let request = decoder.decode(&mut message).unwrap();

                // Post-condition.
                assert_eq!(request, Some(expected_request), "{}", reason);
                assert!(message.is_empty(), "{}", reason);
            });
    }

    #[test]
    fn succeeds_to_encode_response() {
        let cases = vec![
            (
                Response::Get {
                    key: "key".into(),
                    value: None,
                },
                b"FAIL key\n".as_ref(),
                "get without value",
            ),
            (
                Response::Get {
                    key: "key".into(),
                    value: Some("value".into()),
                },
                b"OKAY key value\n".as_ref(),
                "get with value",
            ),
            (
                Response::Set { key: "key".into() },
                b"OKAY key\n".as_ref(),
                "set key",
            ),
        ];

        cases
            .into_iter()
            .for_each(|(response, expected_message, reason)| {
                // Pre-condition.
                let mut encoder = Codec::default();
                let mut message = BytesMut::default();

                // Action.
                encoder.encode(response, &mut message).unwrap();

                // Post-condition.
                assert_eq!(message, expected_message, "{}", reason)
            });
    }

    fn invalid_request_command() -> impl Strategy<Value = String> {
        any::<String>().prop_filter("valid command", |cmd| {
            !vec!["GET", "SET"].contains(&cmd.as_str())
        })
    }
}
