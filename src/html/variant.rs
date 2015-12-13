// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};

use html::{Formatter, where_predicates, angle_generics,  write_raw_type};
use html::markup::{self};
use tree::*;

impl Formatter {
    pub fn variant(&mut self, enum_item: &ItemData, enum_: &Enum, item: &ItemData,
                   variant: &Variant) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Variant "));
        try!(self.h1(&mut file, "Variant "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(syntax(&mut file, enum_item, enum_, item, variant));
        try!(fields(&mut file, item, variant));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }
}

fn syntax<W: Write>(file: &mut W, enum_item: &ItemData, enum_: &Enum, item: &ItemData,
                    variant: &Variant) -> Result {
    try!(file.write_all(b"\
        <h2>Syntax</h2>\
        <pre>\
            enum \
        "));
    try!(file.write_all(enum_item.name.as_ref().unwrap().as_ref()));

    let mut have_where_predicates = enum_.generics.where_predicates.len() > 0;
    have_where_predicates |= try!(angle_generics(file, &enum_.generics));

    if have_where_predicates {
        try!(file.write_all(b"\n"));
        try!(where_predicates(file, &enum_.generics, "    "));
        try!(file.write_all(b"\n{\n"));
    } else {
        try!(file.write_all(b" {\n"));
    }

    try!(file.write_all(b"    "));
    try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
    match variant.kind {
        VariantKind::CLike => { },
        VariantKind::Tuple(ref ts) => {
            try!(file.write_all(b"("));
            let mut first = true;
            for t in ts {
                if !first {
                    try!(file.write_all(b", "));
                }
                first = false;
                try!(write_raw_type(file, t));
            }
            try!(file.write_all(b")"));
        },
        VariantKind::Struct(_) => {
            try!(file.write_all(b"Struct variant documentation not implemented"));
        },
    }

    try!(file.write_all(b"\
            \n}\
        </pre>\
        "));

    Ok(())
}

fn fields<W: Write>(mut file: &mut W, item: &ItemData, variant: &Variant) -> Result {
    let fields = match variant.kind {
        VariantKind::CLike => return Ok(()),
        VariantKind::Tuple(ref f) => f,
        VariantKind::Struct(_) => return Ok(()),
    };

    if fields.len() == 0 {
        return Ok(());
    }

    try!(file.write_all(b"\
        <h2>Fields</h2>\
        <table>\
            <thead>\
                <tr>\
                    <th>Position</th>\
                    <th>Description</th>\
                </tr>\
            </thead>\
            <tbody>\
                "));

    for i in 0..fields.len() {
        try!(file.write_all(b"<tr><td>"));
        try!(write!(file, "{}", i + 1));
        try!(file.write_all(b"</td><td>"));
        let field = try!(format!("{}", i + 1));
        try!(markup::field_desc(file, &item.docs.parts, &field));
        try!(file.write_all(b"</td></tr>"));
    }

    try!(file.write_all(b"\
            </tbody>\
        </table>\
        "));

    Ok(())
}
