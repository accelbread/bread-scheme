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

enum ParseState {
    None,
    Int(i64),
    String(Vec<u8>),
}

pub fn read(input: &mut Input<impl Read>) -> Object {
    let mut state = ParseState::None;
    loop {
        let c = input.get();
        state = match state {
            ParseState::None => match c {
                Some(b'\n' | b' ') => ParseState::None,
                Some(c @ b'0'..=b'9') => ParseState::Int((c - b'0').into()),
                Some(b'"') => ParseState::String(Vec::new()),
                Some(c) => panic!("Parse error: unexpected char: [{}].", c.escape_ascii()),
                None => return Object::Eof,
            },
            ParseState::Int(v) => match c {
                Some(c @ b'0'..=b'9') => ParseState::Int(v * 10 + i64::from(c - b'0')),
                Some(c @ (b' ' | b'\n' | b')')) => {
                    input.push(c);
                    return Object::Int64(v);
                }
                Some(c) => panic!("Parse error: unexpected char [{}].", c.escape_ascii()),
                None => return Object::Int64(v),
            },
            ParseState::String(mut v) => match c {
                Some(b'"') => {
                    return Object::String(
                        String::from_utf8(v)
                            .unwrap_or_else(|e| panic!("Error parsing string: {e}.")),
                    )
                }
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
