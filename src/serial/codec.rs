use std::io::{Error, ErrorKind};
use std::str::{from_utf8, SplitWhitespace};

use serde::Serialize;
use tokio_util::codec::Decoder;

#[derive(Serialize, Default, Debug, Clone, Copy)]
pub struct Reading {
    pub(crate) reference: i64,
    pub(crate) measured: i64,
    pub(crate) displacement: i64,
    pub(crate) velocity: i64,
    pub(crate) zero: i64,
    pub(crate) sequence_num: i64,
    pub(crate) code: i64,
    pub(crate) data: i64,
}

impl TryFrom<&str> for Reading {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let mut iter = s.split_whitespace();

        Ok(Reading {
            reference: next_field("reference", &mut iter)?,
            measured: next_field("measured", &mut iter)?,
            displacement: next_field("displacement", &mut iter)?,
            velocity: next_field("velocity", &mut iter)?,
            zero: next_field("zero", &mut iter)?,
            sequence_num: next_field("sequence_num", &mut iter)?,
            code: next_field("code", &mut iter)?,
            data: next_field("data", &mut iter)?,
        })
    }
}

fn next_field(field: &str, iter: &mut SplitWhitespace<'_>) -> Result<i64, Error> {
    iter.next()
        .ok_or_else(|| Error::new(ErrorKind::InvalidData, format!("Missing field: {}", field)))
        .and_then(|s| {
            s.parse().map_err(|_| {
                Error::new(ErrorKind::InvalidData, format!("Invalid field: {}", field))
            })
        })
}

pub struct LaserCodec;

impl Decoder for LaserCodec {
    type Item = Reading;
    type Error = Error;

    fn decode(
        &mut self,
        src: &mut tokio_util::bytes::BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');

        if let Some(n) = newline {
            let bytes = src.split_to(n + 1);
            let string = from_utf8(&bytes)
                .map_err(|_| Error::new(ErrorKind::Other, "Invalid serial contents"))?;

            return match Reading::try_from(string) {
                Ok(reading) => Ok(Some(reading)),
                Err(e) => Err(e),
            };
        }

        Ok(None)
    }
}
