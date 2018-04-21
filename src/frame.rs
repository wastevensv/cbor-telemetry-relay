extern crate serde;
extern crate serde_cbor;

use serde::ser::{Serialize, Serializer};
use serde_cbor::{ObjectKey, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use proxy_error::ProxyError::{self, ParseError};

#[derive(Serialize, Debug)]
pub struct Frame {
    timestamp: u64,
    sender: String,
    fnum: u64,
    streams: Vec<Stream>,
}

#[derive(Serialize, Debug)]
pub struct Stream {
    name: String,
    value: Data,
    dtype: u64,
}

#[derive(Debug)]
pub enum Data {
    Float(f64),
    Signed(i64),
    Unsigned(u64),
}

impl Serialize for Data {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            &Data::Signed(data) => serializer.serialize_i64(data),
            &Data::Unsigned(data) => serializer.serialize_u64(data),
            &Data::Float(data) => serializer.serialize_f64(data),
        }
    }
}

impl Frame {
    pub fn from_value(val: serde_cbor::Value) -> Result<Self, ProxyError> {
        let map = try!(val.as_object().ok_or(ParseError("Not a map.".to_string())));
        let mut map = map.clone();
        let fnum: Value = try!(
            map.remove(&ObjectKey::String("fnum".to_string()))
                .ok_or(ParseError("No framenumber.".to_string()))
        );
        let fnum: u64 = try!(
            fnum.as_u64()
                .ok_or(ParseError("Invalid framenumber.".to_string()))
        );
        let sender: Value = try!(
            map.remove(&ObjectKey::String("sender".to_string()))
                .ok_or(ParseError("No sender.".to_string()))
        );
        let sender: String = try!(
            sender
                .as_string()
                .ok_or(ParseError("Invalid sender.".to_string()))
                .map(|x| x.to_string())
        );
        let mut streams: Vec<Stream> = vec![];

        for (key, value) in map {
            let key: String = try!(
                key.as_string()
                    .map(|x| x.to_string())
                    .ok_or(ParseError("Invalid key.".to_string()))
            );
            match value {
                Value::F64(value) => streams.push(Stream {
                    name: key,
                    value: Data::Float(value),
                    dtype: 0,
                }),
                Value::I64(value) => streams.push(Stream {
                    name: key,
                    value: Data::Signed(value),
                    dtype: 1,
                }),
                Value::U64(value) => streams.push(Stream {
                    name: key,
                    value: Data::Unsigned(value),
                    dtype: 1,
                }),
                _ => return Err(ParseError(format!("Invalid Value for {}", key))),
            };
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let timestamp = (now.as_secs() * 1e9 as u64) + (now.subsec_nanos() as u64);
        Ok(Frame {
            timestamp,
            streams,
            fnum,
            sender,
        })
    }
}