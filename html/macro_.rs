// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};

use html::{Formatter, markup, angle_generics, where_predicates, write_raw_type, path};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn macro_(&mut self, item: &ItemData, macro_: &Macro) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Macro "));
        try!(self.h1(&mut file, "Macro "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(syntax(&mut file, item, macro_));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));
        Ok(())
    }
}

fn syntax<W: Write>(file: &mut W, item: &ItemData, macro_: &Macro) -> Result {
    try!(file.write_all(b"\
        <h2>Syntax</h2>\
        <pre>\
        "));
    try!(file.write_all(macro_.source.as_ref()));
    try!(file.write_all(b"\
        </pre>\
        "));

    Ok(())
}
