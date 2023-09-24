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

use crate::{input::Input, types::Handle};
use std::io::Read;

#[derive(Default)]
enum ParseState {
    #[default]
    None,
    List(Vec<Handle>),
    MaybeDot(Vec<Handle>),
    ListEnd(Vec<Handle>),
    Int(Vec<u8>),
    Symbol(Vec<u8>),
    String(Vec<u8>),
}

fn make_list(vec: Vec<Handle>) -> Handle {
    let mut iter = vec.into_iter().rev();
    let last = iter.next().unwrap();
    let mut prev = Handle::new_cons(
        match iter.next() {
            Some(e) => e,
            None => return Handle::new_nil(),
        },
        last,
    );
    for e in iter {
        prev = Handle::new_cons(e, prev);
    }
    prev
}

fn make_symbol(vec: Vec<u8>) -> Handle {
    assert!(
        (vec.len() != 1) || (vec[0] != b'.'),
        "Parse error: `.` is not a valid symbol."
    );
    Handle::new_symbol(
        String::from_utf8(vec).unwrap_or_else(|e| panic!("Error parsing identifier: {e}.")),
    )
}

fn make_int(mut v: &[u8]) -> Handle {
    let mut i = 0i64;
    let negative = v[0] == b'-';
    if let b'-' | b'+' = v[0] {
        v = &v[1..];
    }
    for c in v {
        i = i * 10 + i64::from(c - b'0');
    }
    if negative {
        i *= -1;
    }
    Handle::new_int64(i)
}

fn make_string(vec: Vec<u8>) -> Handle {
    Handle::new_string(
        String::from_utf8(vec).unwrap_or_else(|e| panic!("Error parsing identifier: {e}.")),
    )
}

fn is_symbol_char(byte: u8) -> bool {
    matches!(byte,
             b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'!' | b'$' | b'%' | b'&' | b'*' | b'+' |
             b'-' | b'.' | b'/' | b':' | b'<' | b'=' | b'>' | b'?' | b'@' | b'^' | b'_' | b'~')
}

pub fn read(input: &mut Input<impl Read>) -> Handle {
    let mut state = ParseState::None;
    loop {
        let c = input.get();
        state = match state {
            ParseState::None => match c {
                Some(b' ' | b'\t' | b'\n') => ParseState::None,
                Some(b'(') => ParseState::List(Vec::new()),
                Some(b'"') => ParseState::String(Vec::new()),
                Some(b'\'') => {
                    return make_list(vec![
                        Handle::new_symbol("quote".to_string()),
                        read(input),
                        Handle::new_nil(),
                    ]);
                }
                Some(b')') => panic!("Error parsing: unexpected `)`."),
                Some(c @ (b'0'..=b'9' | b'-' | b'+')) => ParseState::Int(vec![c]),
                Some(c) if is_symbol_char(c) => ParseState::Symbol(vec![c]),
                Some(c) => panic!("Error parsing: unexpected `{}`.", c.escape_ascii()),
                None => return Handle::new_eof(),
            },
            ParseState::List(mut v) => match c {
                Some(b'\n' | b' ') => ParseState::List(v),
                Some(b')') => {
                    v.push(Handle::new_nil());
                    return make_list(v);
                }
                Some(b'.') => ParseState::MaybeDot(v),
                Some(c) => {
                    input.push(c);
                    v.push(read(input));
                    ParseState::List(v)
                }
                None => panic!("Error parsing list: unexpected EOF."),
            },
            ParseState::MaybeDot(mut v) => match c {
                Some(b' ' | b'\t' | b'\n') => {
                    assert!(!v.is_empty(), "Error parsing list: unexpected `.`");
                    v.push(read(input));
                    ParseState::ListEnd(v)
                }
                Some(c) => {
                    input.push(c);
                    input.push(b'.');
                    v.push(read(input));
                    ParseState::List(v)
                }
                None => panic!("Error parsing list: unexpected EOF."),
            },
            ParseState::ListEnd(v) => match c {
                Some(b' ' | b'\t' | b'\n') => ParseState::ListEnd(v),
                Some(b')') => return make_list(v),
                Some(_) => panic!("Error parsing list: expected `)`."),
                None => panic!("Error parsing list: unexpected EOF."),
            },
            ParseState::Int(mut v) => match c {
                Some(c @ (b' ' | b'\t' | b'\n' | b'(' | b')')) => {
                    input.push(c);
                    return make_int(&v);
                }
                Some(c @ b'0'..=b'9') => {
                    v.push(c);
                    ParseState::Int(v)
                }
                Some(c) if is_symbol_char(c) => {
                    v.push(c);
                    ParseState::Symbol(v)
                }
                Some(c) => panic!("Error parsing: unexpected `{}`.", c.escape_ascii()),
                None => return make_int(&v),
            },
            ParseState::Symbol(mut v) => match c {
                Some(c @ (b' ' | b'\t' | b'\n' | b'(' | b')')) => {
                    input.push(c);
                    return make_symbol(v);
                }
                Some(c) if is_symbol_char(c) => {
                    v.push(c);
                    ParseState::Symbol(v)
                }
                Some(c) => panic!("Error parsing: unexpected `{}`.", c.escape_ascii()),
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    fn read_str(input: &str) -> Handle {
        read(&mut Input::new(&mut Cursor::new(input)))
    }

    #[test]
    fn eof() {
        assert_eq!(read_str(""), Handle::new_eof());
        assert_eq!(read_str("   "), Handle::new_eof());
        assert_eq!(read_str(" \n"), Handle::new_eof());
    }

    #[test]
    fn list() {
        assert_eq!(read_str("()"), Handle::new_nil());
        assert_eq!(
            read_str("(1)"),
            Handle::new_cons(Handle::new_int64(1), Handle::new_nil())
        );
        assert_eq!(
            read_str("(1 2)"),
            Handle::new_cons(
                Handle::new_int64(1),
                Handle::new_cons(Handle::new_int64(2), Handle::new_nil())
            )
        );
        assert_eq!(
            read_str("(1  2 . 3)"),
            Handle::new_cons(
                Handle::new_int64(1),
                Handle::new_cons(Handle::new_int64(2), Handle::new_int64(3))
            )
        );
        assert_eq!(
            read_str("(1 .a)"),
            Handle::new_cons(
                Handle::new_int64(1),
                Handle::new_cons(Handle::new_symbol(".a".to_string()), Handle::new_nil())
            )
        );
    }
}
