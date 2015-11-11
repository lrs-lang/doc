// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};
use std::string::{ByteStr};

use html::{Formatter, where_predicates, angle_generics, fn_in_out, write_raw_type,
           write_abi, function};
use html::markup::{self};
use tree::*;

impl Formatter {
    pub fn method(&mut self, impl_: &Impl, item: &ItemData, method: &Method) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Method "));
        try!(self.h1(&mut file, "Method "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(self.method_syntax(&mut file, impl_, item, method));
        try!(function::args(&mut file, &method.decl, &item.docs));
        try!(function::return_value(&mut file, &method.decl, &item.docs));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }

    fn method_syntax<W: Write>(&mut self, file: &mut W, impl_: &Impl,
                               item: &ItemData, method: &Method) -> Result {
        try!(file.write_all(b"\
            <h2>Syntax</h2>\
            <pre>\
                impl\
            "));

        // impl block

        let mut have_where_predicates = impl_.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &impl_.generics));

        try!(file.write_all(b" "));
        try!(write_raw_type(file, &impl_.for_));

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &impl_.generics, "    "));
            try!(file.write_all(b"\n{\n"));
        } else {
            try!(file.write_all(b" {\n"));
        }

        // fn block

        method_syntax(file, method, self.path.last().as_ref().unwrap());

        try!(file.write_all(b"\
                \n}\
            </pre>\
            "));

        Ok(())
    }
}

pub fn method_syntax<W: Write>(file: &mut W, method: &Method, name: &ByteStr) -> Result {
    try!(file.write_all(b"    "));
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
        try!(where_predicates(file, &method.generics, "        "));
    }

    Ok(())
}
