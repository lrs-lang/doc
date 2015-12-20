// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};
use std::string::{ByteStr};

use html::{Formatter, where_predicates, angle_generics, fn_in_out, write_abi, function};
use html::markup::{self};
use tree::*;

impl Formatter {
    pub fn trait_method(&mut self, item: &ItemData, method: &Method) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Trait method "));
        try!(self.h1(&mut file, "Trait method "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(syntax(&mut file, method, item.name.as_ref().unwrap().as_ref()));

        try!(function::args(&mut file, &method.decl, &item.docs));
        try!(function::return_value(&mut file, &method.decl, &item.docs));

        try!(markup::description(&mut file, &item.docs.parts));
        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));

        Ok(())
    }
}


// This is almost the same as method::method_syntax buf without the extra whitespace...
fn syntax<W: Write>(file: &mut W, method: &Method, name: &ByteStr) -> Result {
    try!(file.write_all(b"\
        <h2>Syntax</h2>\
        <pre>\
        "));

    if method.unsaf {
        try!(file.write_all(b"unsafe "));
    }
    if try!(write_abi(file, &method.abi)) {
        try!(file.write_all(b" "));
    }
    try!(file.write_all(b"fn "));
    try!(file.write_all(name.as_ref()));

    let mut have_where_predicates = method.generics.where_predicates.len() > 0;
    have_where_predicates |= try!(angle_generics(file, &method.generics));

    try!(fn_in_out(file, &method.self_, &method.decl));

    if have_where_predicates {
        try!(file.write_all(b"\n"));
        try!(where_predicates(file, &method.generics, "   "));
    }

    try!(file.write_all(b"\
        </pre>\
        "));

    Ok(())
}
