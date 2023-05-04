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

use crate::types::{Handle, Object};

pub fn print(value: &Handle) {
    match *value.borrow() {
        Object::Cons(ref car, ref cdr) => print_cons(car, cdr),
        Object::Nil => print!("()"),
        Object::Symbol(ref x) => print!("{x}"),
        Object::Int64(x) => print!("{x}"),
        Object::String(ref x) => print!("\"{x}\""),
        Object::Eof => (),
    };
}

fn print_cons(car: &Handle, cdr: &Handle) {
    print!("(");
    print(car);
    let mut next = cdr.clone();
    while let Object::Cons(ref car, ref cdr) = *next.clone().borrow() {
        print!(" ");
        print(car);
        next = cdr.clone();
    }
    if let Object::Nil = *next.borrow() {
    } else {
        print!(" . ");
        print(cdr);
    }
    print!(")");
}
