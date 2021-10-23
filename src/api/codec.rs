//! Codec for the wire protocol through which requests/responses are exchanged.
//!
//! The wire protocol is optimized for simplicity, where both request
//! and response are line-delimited and further split by whitespaces into
//! components.
//!
//! # Request
//!
//! - GET
//!     - `GET $key\n`
//! - SET
//!     - `SET $key $value\n`
//!
//! # Response
//!
//! - GET
//!     - OK
//!         - `OKAY $key\n`
//! - SET
//!     - OK
//!         - `OKAY $key $value\n`
//!     - FAIL
//!         - `FAIL $key\n`

use super::types::{Request, Response, Status};
use anyhow::{bail, Context, Result};
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LinesCodec};

#[derive(Default, Debug)]
pub struct Codec {
    lines: LinesCodec,
}

impl Decoder for Codec {
    type Item = Request;

    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.lines
            .decode(src)
            .context("unable to decode request line")?
            .as_deref()
            .map(Request::from_wire)
            .transpose()
            .context("unable to parse request")
    }
}

impl Encoder<Response> for Codec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Response, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.lines
            .encode(item.into_wire(), dst)
            .context("unable to encode response line")
    }
}

impl Request {
    fn from_wire(line: &str) -> Result<Self> {
        let mut components = line.split(' ');

        let command = components.next().context("missing command")?;

        match command {
            "GET" => {
                let key = components
                    .next()
                    .context("missing key from GET command")?
                    .into();

                Ok(Request::Get { key })
            }
            "SET" => {
                let key = components
                    .next()
                    .context("missing key from SET command")?
                    .into();

                let value = components
                    .next()
                    .context("missing value from SET command")?
                    .into();

                Ok(Request::Set { key, value })
            }
            _ => bail!("unrecognized command: {}", command),
        }
    }
}

impl Response {
    fn into_wire(self) -> String {
        let status = self.status().into_wire();
        match self {
            Response::Set { key } => {
                format!("{} {}", status, key)
            }
            Response::Get { key, value } => value
                .map(|value| format!("{} {} {}", status, key, value))
                .unwrap_or_else(|| format!("{} {}", status, key)),
        }
    }
}

impl Status {
    fn into_wire(self) -> &'static str {
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
                let encoded_status = status.into_wire();
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
