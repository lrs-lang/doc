// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};

use html::{Formatter, markup, angle_generics, where_predicates, write_raw_type, path};
use tree::*;

impl Formatter {
    pub fn enum_(&mut self, item: &ItemData, enum_: &Enum) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Enum "));
        try!(self.h1(&mut file, "Enum "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(syntax(&mut file, item, enum_));
        try!(self.enum_variants(&mut file, item, enum_));
        try!(self.type_static_methods(&mut file, item));
        try!(self.type_methods(&mut file, item));
        try!(self.type_trait_impls(&mut file, item));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }

    fn enum_variants<W: Write>(&mut self, file: &mut W, enum_item: &ItemData,
                               enum_: &Enum) -> Result {
        let mut variants: Vec<_> = Vec::new();

        for item in &enum_.variants {
            if let Item::Variant(ref v) = item.inner {
                try!(variants.reserve(1));
                variants.push((item, v));
            }
        }

        if variants.len() == 0 {
            return Ok(());
        }

        variants.sort_by(|&(i1, _), &(i2, _)| i1.name.as_ref().unwrap()
                                         .cmp(i2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Variants</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, variant) in &variants {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().try_to()));
            try!(self.variant(enum_item, enum_, item, variant));

            try!(file.write_all(b"\
                <tr>\
                    <td>\
                        <a href=\"./\
                    "));
            try!(file.write_all(try!(path::path(&self.path)).as_ref()));
            try!(file.write_all(b"\">"));
            try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
            try!(file.write_all(b"\
                        </a>\
                    </td>\
                    <td>\
                    "));
            try!(markup::short(file, &item.docs.parts));
            try!(file.write_all(b"\
                    </td>\
                </tr>\
                "));

            self.path.pop();
        }

        try!(file.write_all(b"\
                </tbody>\
            </table>\
            "));

        Ok(())
    }
}

fn syntax<W: Write>(file: &mut W, item: &ItemData, enum_: &Enum) -> Result {
    let mut variants: Vec<_> = Vec::new();

    for item in &enum_.variants {
        if let Item::Variant(ref v) = item.inner {
            try!(variants.reserve(1));
            variants.push((item, v));
        }
    }

    variants.sort_by(|&(i1, _), &(i2, _)| i1.name.as_ref().unwrap()
                                     .cmp(i2.name.as_ref().unwrap()));

    try!(file.write_all(b"\
        <h2>Syntax</h2>\
        <pre>\
            enum \
        "));
    try!(file.write_all(item.name.as_ref().unwrap().as_ref()));

    let mut have_where_predicates = enum_.generics.where_predicates.len() > 0;
    have_where_predicates |= try!(angle_generics(file, &enum_.generics));

    if have_where_predicates {
        try!(file.write_all(b"\n"));
        try!(where_predicates(file, &enum_.generics, "    "));
        try!(file.write_all(b"\n{\n"));
    } else {
        try!(file.write_all(b" {\n"));
    }

    for &(item, variant) in &variants {
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
        try!(file.write_all(b",\n"));
    }

    try!(file.write_all(b"\
            }\
        </pre>\
        "));

    Ok(())
}
