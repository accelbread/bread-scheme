// bread-scheme -- R7RS Scheme interpreter
// Copyright (C) 2023 Archit Gupta <archit@accelbread.com>
//
// This program is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more
// details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(clippy::vec_box)]

use crate::types::Object;
use std::{
    io::{BufReader, ErrorKind, Read},
    slice,
};

pub struct Input<'a, S: Read> {
    stream: BufReader<&'a mut S>,
    buf: Option<u8>,
}

impl<'a, S: Read> Input<'a, S> {
    pub fn new(stream: &'a mut S) -> Self {
        Self {
            stream: BufReader::new(stream),
            buf: None,
        }
    }

    fn get(&mut self) -> Option<u8> {
        if let Some(b) = self.buf {
            self.buf = None;
            Some(b)
        } else {
            let mut b = 0u8;
            match self.stream.read_exact(slice::from_mut(&mut b)) {
                Ok(_) => Some(b),
                Err(e) => match e.kind() {
                    ErrorKind::UnexpectedEof => None,
                    _ => panic!("Input error: {e}"),
                },
            }
        }
    }

    fn push(&mut self, byte: u8) {
        self.buf = match self.buf {
            None => Some(byte),
            Some(_) => panic!("Pushing byte onto input existing pushed byte."),
        };
    }

    pub fn has_pending(&self) -> bool {
        self.buf.is_some() || !self.stream.buffer().is_empty()
    }

    pub fn clear_pending_space(&mut self) {
        while self.has_pending() {
            let c = self.get();
            match c {
                Some(b' ') => (),
                Some(b'\n') => return,
                Some(c) => {
                    self.push(c);
                    return;
                }
                None => unreachable!(),
            };
        }
    }
}

#[derive(Default)]
enum ParseState {
    #[default]
    None,
    List(Vec<Box<Object>>),
    Int(Vec<u8>),
    Symbol(Vec<u8>),
    String(Vec<u8>),
}

fn make_list(vec: Vec<Box<Object>>) -> Object {
    let mut iter = vec.into_iter().rev();
    let mut prev = Object::Cons(
        match iter.next() {
            Some(e) => e,
            None => return Object::Nil,
        },
        Box::new(Object::Nil),
    );
    for e in iter {
        prev = Object::Cons(e, Box::new(prev));
    }
    prev
}

fn make_symbol(vec: Vec<u8>) -> Object {
    Object::Symbol(
        String::from_utf8(vec).unwrap_or_else(|e| panic!("Error parsing identifier: {e}.")),
    )
}

fn make_int(vec: Vec<u8>) -> Object {
    let mut i = 0i64;
    for c in vec {
        i = i * 10 + i64::from(c - b'0');
    }
    Object::Int64(i)
}

fn make_string(vec: Vec<u8>) -> Object {
    Object::String(
        String::from_utf8(vec).unwrap_or_else(|e| panic!("Error parsing identifier: {e}.")),
    )
}

pub fn read(input: &mut Input<impl Read>) -> Object {
    let mut state = ParseState::None;
    loop {
        let c = input.get();
        state = match state {
            ParseState::None => match c {
                Some(b'\n' | b' ') => ParseState::None,
                Some(b'(') => ParseState::List(Vec::new()),
                Some(b'"') => ParseState::String(Vec::new()),
                Some(c @ b'0'..=b'9') => ParseState::Int(vec![c]),
                Some(c) => ParseState::Symbol(vec![c]),
                None => return Object::Eof,
            },
            ParseState::List(mut v) => match c {
                Some(b'\n' | b' ') => ParseState::List(v),
                Some(b')') => return make_list(v),
                Some(c) => {
                    input.push(c);
                    v.push(Box::new(read(input)));
                    ParseState::List(v)
                }
                None => panic!("Error parsing list: unexpected EOF."),
            },
            ParseState::Int(mut v) => match c {
                Some(c @ (b' ' | b'\n' | b')')) => {
                    input.push(c);
                    return make_int(v);
                }
                Some(c @ b'0'..=b'9') => {
                    v.push(c);
                    ParseState::Int(v)
                }
                Some(_) => ParseState::Symbol(v),
                None => return make_int(v),
            },
            ParseState::Symbol(mut v) => match c {
                Some(c @ (b' ' | b'\n' | b')')) => {
                    input.push(c);
                    return make_symbol(v);
                }
                Some(c) => {
                    v.push(c);
                    ParseState::Symbol(v)
                }
                None => return make_symbol(v),
            },
            ParseState::String(mut v) => match c {
                Some(b'"') => return make_string(v),
                Some(b'\\') => ParseState::String(v),
                Some(c) => {
                    v.push(c);
                    ParseState::String(v)
                }
                None => panic!("Error parsing string: unexpected EOF."),
            },
        };
    }
}
