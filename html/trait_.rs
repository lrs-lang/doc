// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};
use lrs::rc::{Arc};
use lrs::vec::{SVec};

use html::{self, Formatter, markup, angle_generics, where_predicates, write_ty_param_bounds, write_raw_type, path};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn trait_(&mut self, item: &ItemData, trait_: &Trait) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Trait "));
        try!(self.h1(&mut file, "Trait "));

        try!(markup::short(&mut file, &item.docs.parts));

        let (mut assocs, mut required, mut provided) = try!(collect_parts(trait_));

        try!(self.trait_syntax(&mut file, trait_, &assocs, &required, &provided));
        try!(assoc_types(&mut file, &mut assocs));

        try!(required.push_all(&provided));
        required.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap().as_ref()
                                         .cmp(f2.name.as_ref().unwrap().as_ref()));

        try!(self.trait_methods(&mut file, &required));
        try!(self.type_trait_impls(&mut file, item));

        try!(markup::remarks(&mut file, &item.docs.parts));
        try!(markup::examples(&mut file, &item.docs.parts));
        try!(markup::see_also(&mut file, &item.docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }

    fn trait_syntax<W: Write>(&self, file: &mut W, trait_: &Trait,
                              assocs: &[(&Arc<ItemData>, &AssocType)],
                              required: &[(&Arc<ItemData>, &Method)],
                              provided: &[(&Arc<ItemData>, &Method)],
                              ) -> Result {
        try!(file.write_all(b"\
            <h2>Syntax</h2>\
            <pre>\
            "));
        if trait_.unsaf {
            try!(file.write_all(b"unsafe "));
        }
        try!(file.write_all(b"trait "));
        try!(file.write_all(self.path[self.path.len()-1].as_ref()));

        let mut have_where_predicates = trait_.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &trait_.generics));

        if trait_.bounds.len() > 0 {
            try!(file.write_all(b" : "));
            try!(write_ty_param_bounds(file, &trait_.bounds));
        }

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &trait_.generics, "    "));
        }

        if assocs.len() + required.len() + provided.len() > 0 {
            if have_where_predicates {
                try!(file.write_all(b"\n{\n"));
            } else {
                try!(file.write_all(b" {\n"));
            }

            for &(i, a) in assocs {
                try!(file.write_all(b"    type "));
                try!(file.write_all(i.name.as_ref().unwrap().as_ref()));
                if a.bounds.len() > 0 {
                    try!(file.write_all(b": "));
                    try!(write_ty_param_bounds(file, &a.bounds));
                }
                try!(file.write_all(b";\n"));
            }

            if required.len() > 0 {
                if assocs.len() > 0 {
                    try!(file.write_all(b"\n"));
                }
                try!(file.write_all(b"    /* Required methods */\n"));
            }

            for &(i, m) in required {
                html::method::method_syntax(file, m, i.name.as_ref().unwrap());
                try!(file.write_all(b"\n"));
            }

            if provided.len() > 0 {
                if assocs.len() + required.len() > 0 {
                    try!(file.write_all(b"\n"));
                }
                try!(file.write_all(b"    /* Provided methods */\n"));
            }

            for &(i, m) in provided {
                html::method::method_syntax(file, m, i.name.as_ref().unwrap());
                try!(file.write_all(b"\n"));
            }

            try!(file.write_all(b"}"));
        }

        try!(file.write_all(b"\
            </pre>\
            "));

        Ok(())
    }

    fn trait_methods<W: Write>(&mut self, file: &mut W,
                               methods: &[(&Arc<ItemData>, &Method)]) -> Result {
        if methods.len() == 0 {
            return Ok(());
        }

        try!(file.write_all(b"\
            <h2>Methods</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Receiver</th>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, method) in methods {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.trait_method(item, method));

            try!(file.write_all(b"\
                <tr>\
                    <td><code class=\"no_break\">\
                    "));
            match method.self_ {
                SelfTy::Static => { },
                SelfTy::Value => { try!(file.write_all(b"self")); }
                SelfTy::Borrowed(ref lt, mutable) => {
                    try!(file.write_all(b"&"));
                    if mutable {
                        try!(file.write_all(b"mut "));
                    }
                    try!(file.write_all(b"self"));
                },
                SelfTy::Explicit(ref t) => { try!(write_raw_type(file, t)); }
            }
            try!(file.write_all(b"\
                    </code></td>\
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

fn assoc_types<W: Write>(file: &mut W,
                         assocs: &[(&Arc<ItemData>, &AssocType)]) -> Result {
    if assocs.len() == 0 {
        return Ok(());
    }

    try!(file.write_all(b"\
        <h2>Associated types</h2>\
        <table>\
            <thead>\
                <tr>\
                    <th>Name</th>\
                    <th>Description</th>\
                </tr>\
            </thead>\
            <tbody>\
                "));

    for &(a, _) in assocs {
        try!(file.write_all(b"\
            <tr>\
                <td>\
                "));
        try!(file.write_all(a.name.as_ref().unwrap().as_ref()));
        try!(file.write_all(b"\
                </td>\
                <td>\
                "));
        try!(markup::all(file, &a.docs.parts));
        try!(file.write_all(b"\
                </td>\
            </tr>\
            "));
    }

    try!(file.write_all(b"\
            </tbody>\
        </table>\
        "));

    Ok(())
}


type Res<'a> = (SVec<(&'a Arc<ItemData>, &'a AssocType)>,
                SVec<(&'a Arc<ItemData>, &'a Method)>,
                SVec<(&'a Arc<ItemData>, &'a Method)>);
            

fn collect_parts(trait_: &Trait) -> Result<Res> {
    let mut assoc: Vec<_> = Vec::new();
    let mut decl: Vec<_> = Vec::new();
    let mut method: Vec<_> = Vec::new();

    for item in &trait_.items {
        match item.inner {
            Item::AssocType(ref a) => {
                try!(assoc.reserve(1));
                assoc.push((item, a));
            },
            Item::MethodDecl(ref m) => {
                try!(decl.reserve(1));
                decl.push((item, m));
            },
            Item::Method(ref m) => {
                try!(method.reserve(1));
                method.push((item, m));
            },
            _ => { },
        }
    }

    assoc.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap().as_ref()
                                  .cmp(f2.name.as_ref().unwrap().as_ref()));
    decl.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap().as_ref()
                                 .cmp(f2.name.as_ref().unwrap().as_ref()));
    method.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap().as_ref()
                                   .cmp(f2.name.as_ref().unwrap().as_ref()));

    Ok((assoc, decl, method))
}
