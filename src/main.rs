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

//! R7RS Scheme interpreter

#![warn(missing_docs, clippy::pedantic, clippy::cargo)]
#![allow(clippy::similar_names)]

mod parser;
mod types;

use crate::parser::read;
use crate::types::Object;
use std::{
    io::{self, Write},
    process::exit,
};

fn eval(value: &Object) -> Object {
    value.clone()
}

fn print(value: &Object) {
    match value {
        Object::Cons(car, cdr) => print_cons(car, cdr),
        Object::Nil => print!("()"),
        Object::Symbol(x) => print!("{x}"),
        Object::Int64(x) => print!("{x}"),
        Object::String(x) => print!("\"{x}\""),
        Object::Eof => (),
    };
}

fn print_cons(car: &Object, mut cdr: &Object) {
    print!("(");
    print(car);
    while let Object::Cons(cdar, cddr) = cdr {
        print!(" ");
        print(cdar);
        cdr = cddr;
    }
    if let Object::Nil = cdr {
    } else {
        print!(" . ");
        print(cdr);
    }
    print!(")");
}

fn main() {
    println!("Welcome to Bread Scheme!");
    let mut handle = &mut io::stdin().lock();
    let mut input = parser::Input::new(&mut handle);
    loop {
        if !input.has_pending() {
            print!(">>> ");
            let _ = io::stdout().flush();
        }
        let parsed = &read(&mut input);
        if let Object::Eof = parsed {
            exit(0);
        }
        print(&eval(parsed));
        println!();
        input.clear_pending_space();
    }
}
