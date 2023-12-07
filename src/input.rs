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

use std::{
    io::{BufReader, ErrorKind, Read},
    slice,
};

pub struct Input<'a, S: Read> {
    stream: BufReader<&'a mut S>,
    buffered: usize,
    buf: [u8; 2],
}

impl<'a, S: Read> Input<'a, S> {
    pub fn new(stream: &'a mut S) -> Self {
        Self {
            stream: BufReader::new(stream),
            buffered: 0,
            buf: [0, 0],
        }
    }

    pub fn get(&mut self) -> Option<u8> {
        if self.buffered > 0 {
            let c = self.buf[0];
            self.buffered -= 1;
            self.buf[0] = std::mem::take(&mut self.buf[1]);
            Some(c)
        } else {
            let mut c = 0u8;
            match self.stream.read_exact(slice::from_mut(&mut c)) {
                Ok(()) => Some(c),
                Err(e) => match e.kind() {
                    ErrorKind::UnexpectedEof => None,
                    _ => panic!("Input error: {e}"),
                },
            }
        }
    }

    pub fn push(&mut self, byte: u8) {
        assert!(
            self.buffered < self.buf.len(),
            "Pushing byte onto input with no space."
        );

        self.buffered += 1;
        self.buf[1] = self.buf[0];
        self.buf[0] = byte;
    }

    pub fn has_pending(&self) -> bool {
        self.buffered > 0 || !self.stream.buffer().is_empty()
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
