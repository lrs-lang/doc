// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};
use lrs::string::{ByteString};
use lrs::iter::{IteratorExt};

use html::{Formatter, where_predicates, angle_generics, write_raw_type};
use html::markup::{self};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn struct_(&mut self, item: &ItemData, strukt: &Struct,
                   docs: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Struct "));
        try!(self.h1(&mut file, "Struct "));

        try!(markup::short(&mut file, &docs.parts));

        try!(self.struct_syntax(&mut file, strukt));
        try!(self.struct_fields(&mut file, strukt, docs));
        try!(self.type_static_methods(&mut file, item));

        try!(markup::remarks(&mut file, &docs.parts));
        try!(markup::see_also(&mut file, &docs.parts));

        try!(self.foot(&mut file));
        Ok(())
    }

    fn struct_syntax<W: Write>(&mut self, file: &mut W, strukt: &Struct) -> Result {
        try!(file.write_all(b"\
            <h2>Syntax</h2>\
            <pre>\
                struct \
            "));
        try!(file.write_all(self.path[self.path.len()-1].as_ref()));

        let mut have_where_predicates = strukt.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &strukt.generics));

        if strukt.struct_type == StructType::Tuple {
            try!(file.write_all(b"("));
            let mut first = true;
            for item in &strukt.fields {
                if !first {
                    try!(file.write_all(b", "));
                }
                first = false;
                let field = match item.inner {
                    Item::StructField(ref f) => f,
                    _ => errexit!("struct field is not a StructField"),
                };
                match *field {
                    StructField::Hidden => {
                        try!(file.write_all(b"/* */"));
                    },
                    StructField::Typed(ref t) => {
                        try!(write_raw_type(file, t));
                    },
                }
            }
            try!(file.write_all(b")"));
        }

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &strukt.generics, "    "));
        }

        if strukt.struct_type == StructType::Plain {
            if have_where_predicates {
                try!(file.write_all(b"\n{\n"));
            } else {
                try!(file.write_all(b" {\n"));
            }
            let mut have_hidden = false;
            for item in &strukt.fields {
                let field = match item.inner {
                    Item::StructField(ref f) => f,
                    _ => errexit!("struct field is not a StructField"),
                };
                if let StructField::Typed(ref t) = *field {
                    try!(file.write_all(b"    "));
                    try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
                    try!(file.write_all(b": "));
                    try!(write_raw_type(file, t));
                    try!(file.write_all(b",\n"));
                } else {
                    have_hidden = true;
                }
            }
            if have_hidden {
                try!(file.write_all(b"    /* private fields */\n"));
            }
            try!(file.write_all(b"}"));
        }

        try!(file.write_all(b"\
            </pre>\
            "));

        Ok(())
    }

    fn struct_fields<W: Write>(&mut self, mut file: &mut W, strukt: &Struct,
                               docs: &Document) -> Result {
        let mut have_public_fields = false;
        for field in &strukt.fields {
            if let Item::StructField(ref f) = field.inner {
                if let StructField::Typed(_) = *f {
                    have_public_fields = true;
                    break;
                }
            }
        }

        if !have_public_fields {
            return Ok(());
        }

        try!(file.write_all(b"\
            <h2>Fields</h2>\
            <table>\
                <thead>\
                    <tr>\
                    "));
        if strukt.struct_type == StructType::Tuple {
            try!(file.write_all(b"<th>Position</th>"));
        } else {
            try!(file.write_all(b"<th>Name</th>"));
        }
        try!(file.write_all(b"\
                        <th>Type</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for (i, item) in strukt.fields.iter().enumerate() {
            let field = match item.inner {
                Item::StructField(ref f) => f,
                _ => errexit!("struct field is not a StructField"),
            };
            let t = match *field {
                StructField::Typed(ref t) => t,
                _ => continue,
            };
            try!(file.write_all(b"<tr><td>"));
            if strukt.struct_type == StructType::Tuple {
                try!(write!(file, "{}", i + 1));
            } else {
                try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
            }
            try!(file.write_all(b"</td><td><code>"));
            try!(write_raw_type(file, t));
            try!(file.write_all(b"</code></td><td>"));
            if strukt.struct_type == StructType::Tuple {
                let field: ByteString = format!("{}", i + 1);
                try!(markup::field_desc(file, &docs.parts, field.as_ref()));
            } else {
                try!(markup::all(file, &item.docs.parts));
            }
            try!(file.write_all(b"</td></tr>"));
        }

        try!(file.write_all(b"\
                </tbody>\
            </table>\
            "));

        Ok(())
    }
}
