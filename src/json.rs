// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Simple Json parser
//!
//! Not supported: Floating point numbers

use std::error::{Errno, InvalidArgument};
use std::{mem};
use std::vec::{Vec};
use std::io::{BufRead};
use std::parse::{Parsable};

macro_rules! error {
    ($fmt:expr) => { error!(concat!($fmt, "{}"), "") };
    ($fmt:expr, $($arg:tt)*) => {{
        errln!(concat!("lrs_doc: Error: ", $fmt), $($arg)*);
        return Err(ERR);
    }};
}

const ERR: Errno = InvalidArgument;

pub type Object = Vec<(Vec<u8>, Value)>;
pub type Array = Vec<Value>;
pub type Slice = [Value];
pub enum Value {
    String(Vec<u8>),
    Integer(i64),
    Object(Object),
    Array(Array),
    Boolean(bool),
    Null,
}

pub fn parse(mut data: &[u8]) -> Result<Value> {
    Ok(Value::Object(try!(object(&mut data))))
}

fn whitespace(data: &mut &[u8]) {
    while data.len() > 0 {
        match data[0] {
            b' ' | b'\t' | b'\n' | b'\r' => { data.consume(1); },
            _ => break,
        }
    }
}

fn value(data: &mut &[u8]) -> Result<Value> {
    whitespace(data);
    if data.len() == 0 { error!("Value has length 0"); }
    let value = match data[0] {
        b'"'               => Value::String(try!(string(data))),
        b'-' | b'0'...b'9' => Value::Integer(try!(integer(data))),
        b'{'               => Value::Object(try!(object(data))),
        b'['               => Value::Array(try!(array(data))),
        b't' | b'f'        => Value::Boolean(try!(boolean(data))),
        b'n'               => try!(null(data)),
        _ => error!("Value starts with unknown letter: {:?}", data[0] as char),
    };
    Ok(value)
}

fn object(data: &mut &[u8]) -> Result<Object> {
    let mut pairs = Vec::new();

    whitespace(data);
    if data.len() == 0 || data[0] != b'{' { error!("Object is empty or doesn't start with {{"); }
    data.consume(1);
    whitespace(data);
    if data.len() != 0 && data[0] == b'}' {
        data.consume(1);
        return Ok(pairs);
    }

    loop {
        let key = try!(string(data));
        whitespace(data);
        if data.len() == 0 || data[0] != b':' { error!("Can't find : in object"); }
        data.consume(1);
        let value = try!(value(data));
        try!(pairs.reserve(1));
        pairs.push((key, value));
        whitespace(data);
        if data.len() == 0 { error!("Can't find , or }} in object"); }
        if data[0] == b'}' { break; }
        if data[0] == b',' { data.consume(1); }
    }

    data.consume(1);
    Ok(pairs)
}

fn string(data: &mut &[u8]) -> Result<Vec<u8>> {
    whitespace(data);
    if data.len() == 0 || data[0] != b'"' { error!("String is empty or doesn't start with \""); }
    data.consume(1);
    let mut string = Vec::new();
    let mut escape = false;

    loop {
        if data.len() == 0 { error!("Unterminated string"); }
        if !escape && data[0] == b'"' { break; }
        try!(string.reserve(1));

        if escape {
            escape = false;

            match data[0] {
                b'"' => string.push(b'"'),
                b'\\' => string.push(b'\\'),
                b'/' => string.push(b'/'),
                b'b' => string.push(8),
                b'f' => string.push(12),
                b'n' => string.push(10),
                b'r' => string.push(13),
                b't' => string.push(b'\t'),
                b'u' => {
                    data.consume(1);
                    if data.len() < 4 { error!("Unicode escape sequence too short"); }
                    let mut num = *b"0x0000";
                    mem::copy(&mut num[2..], &data[..4]);
                    data.consume(4);

                    let c = match char::from_u32(try!(num.parse())) {
                        Some(c) => c,
                        None => error!("Unicode escape sequence invalid"),
                    };
                    let encoded = c.to_utf8();
                    let len = c.len();
                    try!(string.reserve(len));
                    string.push_all(&encoded[..len]);

                    continue;
                },
                _ => error!("Unknown escape character: {:?}", data[0] as char),
            }
        } else if data[0] == b'\\' {
            escape = true;
        } else {
            string.push(data[0]);
        }

        data.consume(1);
    }

    data.consume(1);
    Ok(string)
}

fn integer(data: &mut &[u8]) -> Result<i64> {
    whitespace(data);
    let (num, len) = try!(i64::parse_bytes_init(*data));
    data.consume(len);
    Ok(num)
}

fn array(data: &mut &[u8]) -> Result<Array> {
    let mut array = Vec::new();

    whitespace(data);
    if data.len() == 0 || data[0] != b'[' { error!("Array doesn't start with ["); }
    data.consume(1);
    whitespace(data);
    if data.len() != 0 && data[0] == b']' {
        data.consume(1);
        return Ok(array);
    }

    loop {
        let value = try!(value(data));
        try!(array.reserve(1));
        array.push(value);
        whitespace(data);
        if data.len() == 0 { error!("Can't find , or ] in array"); }
        if data[0] == b']' { break; }
        if data[0] == b',' { data.consume(1); }
    }

    data.consume(1);
    Ok(array)
}

fn boolean(data: &mut &[u8]) -> Result<bool> {
    const TRUE: &'static [u8] = b"true";
    const FALSE: &'static [u8] = b"false";

    whitespace(data);
    if data.len() >= TRUE.len() && &data[..TRUE.len()] == TRUE {
        data.consume(TRUE.len());
        Ok(true)
    } else if data.len() >= FALSE.len() && &data[..FALSE.len()] == FALSE {
        data.consume(FALSE.len());
        Ok(false)
    } else {
        error!("Can't parse boolean");
    }
}

fn null(data: &mut &[u8]) -> Result<Value> {
    const NULL: &'static [u8] = b"null";

    whitespace(data);
    if data.len() >= NULL.len() && &data[..NULL.len()] == NULL {
        data.consume(NULL.len());
        Ok(Value::Null)
    } else {
        error!("Can't parse null");
    }
}
