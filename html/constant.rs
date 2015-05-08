// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};

use html::{Formatter, markup, angle_generics, where_predicates, write_raw_type, path};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn constant(&mut self, item: &ItemData, constant: &Constant) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Constant "));
        try!(self.h1(&mut file, "Constant "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(syntax(&mut file, item, constant));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }
}

fn syntax<W: Write>(file: &mut W, item: &ItemData, constant: &Constant) -> Result {
    try!(file.write_all(b"\
        <h2>Syntax</h2>\
        <pre>\
            const \
        "));
    try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
    try!(file.write_all(b": "));
    try!(write_raw_type(file, &constant.type_));
    try!(file.write_all(b" = "));
    try!(file.write_all(constant.expr.as_ref()));
    try!(file.write_all(b"\
            ;\
        </pre>\
        "));

    Ok(())
}

