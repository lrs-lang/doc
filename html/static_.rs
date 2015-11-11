// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};

use html::{Formatter, markup, angle_generics, where_predicates, write_raw_type, path};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn static_(&mut self, item: &ItemData, static_: &Static) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Static "));
        try!(self.h1(&mut file, "Static "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(syntax(&mut file, item, static_));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }
}

fn syntax<W: Write>(file: &mut W, item: &ItemData, static_: &Static) -> Result {
    try!(file.write_all(b"\
        <h2>Syntax</h2>\
        <pre>\
            static \
        "));
    if static_.mutable {
        try!(file.write_all(b"mut "));
    }
    try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
    try!(file.write_all(b": "));
    try!(write_raw_type(file, &static_.type_));
    try!(file.write_all(b" = "));
    try!(file.write_all(static_.expr.as_ref()));
    try!(file.write_all(b"\
            ;\
        </pre>\
        "));

    Ok(())
}


