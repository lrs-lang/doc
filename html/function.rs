// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};

use html::{Formatter, write_abi, angle_generics, fn_in_out, where_predicates, markup};
use tree::*;

impl Formatter {
    pub fn function(&mut self, item: &ItemData, func: &Func) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Function "));
        try!(self.h1(&mut file, "Function "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(self.function_syntax(&mut file, func));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));
        Ok(())
    }

    fn function_syntax<W: Write>(&mut self, file: &mut W, func: &Func) -> Result {
        try!(file.write_all(b"\
            <h2>Syntax</h2>\
            <pre>\
            "));
        if func.unsaf {
            try!(file.write_all(b"unsafe "));
        }
        if try!(write_abi(file, &func.abi)) {
            try!(file.write_all(b" "));
        }
        try!(file.write_all(b"fn "));
        try!(file.write_all(self.path.last().as_ref().unwrap().as_ref()));

        let mut have_where_predicates = func.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &func.generics));

        try!(fn_in_out(file, &SelfTy::Static, &func.decl));

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &func.generics, "    "));
        }

        try!(file.write_all(b"\
                \n}\
            </pre>\
            "));

        Ok(())
    }
}
