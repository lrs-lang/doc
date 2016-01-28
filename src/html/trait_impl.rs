// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};
use std::rc::{Arc};

use html::{Formatter, where_predicates, angle_generics, write_raw_type};
use html::markup::{self};
use tree::*;

type Input<'a> = (&'a Arc<ItemData>, &'a Impl, Arc<ItemData>, &'a Type);

impl Formatter {
    pub fn trait_impl(&mut self, impls: &[Input]) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Trait implementation "));
        try!(self.h1(&mut file, "Trait implementation "));

        if impls.len() > 1 {
            try!(file.write_all(b"\
                    <p>\
                        <b>\
                            Note: This page contains multiple implementations.\
                        </b>\
                    </p>\
                    "));
        }

        try!(markup::short(&mut file, &impls[0].2.docs.parts));

        for &(impl_item, impl_, ref trait_item, trait_) in impls {
            try!(self.trait_impl_syntax(&mut file, impl_item, impl_, trait_item, trait_));

            try!(markup::description(&mut file, &impl_item.docs.parts));
            try!(markup::remarks(&mut file, &impl_item.docs.parts));
            try!(markup::examples(&mut file, &impl_item.docs.parts));
            try!(markup::see_also(&mut file, &impl_item.docs.parts));
        }

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }

    fn trait_impl_syntax<W: Write>(&mut self, file: &mut W, _item_impl: &Arc<ItemData>,
                                   impl_: &Impl, _trait_item: &Arc<ItemData>,
                                   trait_: &Type) -> Result {
        try!(file.write_all(b"\
            <h2>Syntax</h2>\
            <pre>\
                impl\
            "));

        let mut have_where_predicates = impl_.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &impl_.generics));

        if impl_.negative == Some(true) {
            try!(file.write_all(b" !"));
        } else {
            try!(file.write_all(b" "));
        }
        try!(write_raw_type(file, trait_));
        try!(file.write_all(b" for "));
        try!(write_raw_type(file, &impl_.for_));

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &impl_.generics, "    "));
        }

        let mut assocs: Vec<_> = Vec::new();

        for item in &impl_.items {
            if let Item::Typedef(ref a) = item.inner {
                try!(assocs.push((item, a)));
            }
        }

        if assocs.len() > 0 {
            if have_where_predicates {
                try!(file.write_all(b"\n{\n"));
            } else {
                try!(file.write_all(b" {\n"));
            }
            for &(item, assoc) in &assocs {
                try!(file.write_all(b"    type "));
                try!(file.write_all(item.name.as_ref().unwrap().as_ref()));
                try!(file.write_all(b" = "));
                try!(write_raw_type(file, &assoc.type_));
                try!(file.write_all(b";\n"));
            }
            try!(file.write_all(b"}"));
        }

        try!(file.write_all(b"\
            </pre>\
            "));

        Ok(())
    }

}
