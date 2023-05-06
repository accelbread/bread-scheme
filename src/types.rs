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

use std::{
    cell::{Ref, RefCell},
    fmt::{self, Display},
    rc::Rc,
};

#[derive(Clone, Default, PartialEq, Eq)]
pub enum Object {
    #[default]
    Empty,
    Cons(Handle, Handle),
    Symbol(String),
    Int64(i64),
    String(String),
    Eof,
}

// Freeing reference cycles is a future problem

#[derive(Clone, PartialEq, Eq)]
pub struct Handle(Rc<RefCell<Object>>);

impl Handle {
    pub fn new_nil() -> Self {
        Handle(Rc::new(RefCell::new(Object::Empty)))
    }

    pub fn new_cons(car: Handle, cdr: Handle) -> Self {
        Handle(Rc::new(RefCell::new(Object::Cons(car, cdr))))
    }

    pub fn new_symbol(value: String) -> Self {
        Handle(Rc::new(RefCell::new(Object::Symbol(value))))
    }

    pub fn new_int64(value: i64) -> Self {
        Handle(Rc::new(RefCell::new(Object::Int64(value))))
    }

    pub fn new_string(value: String) -> Self {
        Handle(Rc::new(RefCell::new(Object::String(value))))
    }

    pub fn new_eof() -> Self {
        Handle(Rc::new(RefCell::new(Object::Eof)))
    }

    pub fn borrow(&self) -> Ref<Object> {
        self.0.borrow()
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self.borrow() {
            Object::Cons(ref car, ref cdr) => write_cons(car, cdr, f),
            Object::Empty => write!(f, "()"),
            Object::Symbol(ref x) => write!(f, "{x}"),
            Object::Int64(x) => write!(f, "{x}"),
            Object::String(ref x) => write!(f, "\"{x}\""),
            Object::Eof => write!(f, "#<eof>"),
        }
    }
}

impl fmt::Debug for Handle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

fn write_cons(car: &Handle, cdr: &Handle, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "(")?;
    car.fmt(f)?;
    let mut next = cdr.clone();
    while let Object::Cons(ref car, ref cdr) = *next.clone().borrow() {
        write!(f, " ")?;
        car.fmt(f)?;
        next = cdr.clone();
    }
    if let Object::Empty = *next.borrow() {
    } else {
        write!(f, " . ")?;
        next.fmt(f)?;
    }
    write!(f, ")")
}
