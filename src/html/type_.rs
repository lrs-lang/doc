// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};
use std::iter::{IteratorExt};
use std::clone::{MaybeClone};

use html::{path, Formatter, write_raw_type};
use html::markup::{self};
use tree::*;

impl Formatter {
    pub fn type_(&mut self, item: &ItemData) -> Result {
        match item.inner {
            Item::Struct(ref  s) => self.struct_(item, s),
            Item::Enum(ref    e) => self.enum_(item, e),
            Item::Typedef(ref t) => self.typedef(item, t),
            Item::Trait(ref   t) => self.trait_(item, t),
            _ => abort!(),
        }
    }

    pub fn type_static_methods<W: Write>(&mut self, mut file: &mut W,
                                         item: &ItemData) -> Result {
        let impls = item.impls.borrow();

        let mut methods: Vec<_> = Vec::new();

        for impl_item in &*impls {
            if let Item::Impl(ref impl_) = impl_item.inner {
                if impl_.trait_.is_none() {
                    for item in &impl_.items {
                        if let Item::Method(ref method) = item.inner {
                            if let SelfTy::Static = method.self_ {
                                try!(methods.reserve(1));
                                methods.push((impl_, item, method));
                            }
                        }
                    }
                }
            }
        }

        if methods.len() == 0 {
            return Ok(());
        }

        methods.sort_by(|&(_, i1, _), &(_, i2, _)| i1.name.as_ref().unwrap()
                                              .cmp(i2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Static methods</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(impl_, item, method) in &methods {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().maybe_clone()));
            try!(self.method(impl_, item, method));

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

    pub fn type_methods<W: Write>(&mut self, mut file: &mut W,
                                    item: &ItemData) -> Result {
        let impls = item.impls.borrow();

        let mut methods: Vec<_> = Vec::new();

        for impl_item in &*impls {
            if let Item::Impl(ref impl_) = impl_item.inner {
                if impl_.trait_.is_none() {
                    for item in &impl_.items {
                        if let Item::Method(ref method) = item.inner {
                            match method.self_ {
                                SelfTy::Static => { },
                                _ => {
                                    try!(methods.reserve(1));
                                    methods.push((impl_, item, method));
                                },
                            }
                        }
                    }
                }
            }
        }

        if methods.len() == 0 {
            return Ok(());
        }

        methods.sort_by(|&(_, i1, _), &(_, i2, _)| i1.name.as_ref().unwrap()
                                              .cmp(i2.name.as_ref().unwrap()));

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

        for &(impl_, item, method) in &methods {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().maybe_clone()));
            try!(self.method(impl_, item, method));

            try!(file.write_all(b"\
                <tr>\
                    <td><code class=\"no_break\">\
                    "));
            match method.self_ {
                SelfTy::Static => { },
                SelfTy::Value => { try!(file.write_all(b"self")); }
                SelfTy::Borrowed(_, mutable) => {
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

    pub fn type_trait_impls<W: Write>(&mut self, mut file: &mut W,
                                      item: &ItemData) -> Result {
        let all_impls = item.impls.borrow();

        let mut impls: Vec<_> = Vec::new();

        for impl_item in &*all_impls {
            if let Item::Impl(ref impl_) = impl_item.inner {
                if let Some(ref trait_) = impl_.trait_ {
                    if let Type::ResolvedPath(ref path) = *trait_ {
                        let borrow = path.item.borrow();
                        if let Some(ref trait_item) = *borrow {
                            try!(impls.reserve(1));
                            impls.push((impl_item, impl_, trait_item.add_ref(), trait_));
                        }
                    }
                }
            }
        }

        if impls.len() == 0 {
            return Ok(());
        }

        impls.sort_by(|&(_, _, ref t1, _), &(_, _, ref t2, _)| t1.name.as_ref().unwrap()
                                                          .cmp(t2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Trait implementations</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        let mut num_impls = 1;

        for (i, &(_impl_item, _impl_, ref trait_item, _trait_)) in impls.iter().enumerate() {
            if i + 1 < impls.len() {
                if &*impls[i+1].2 as *const ItemData == &**trait_item as *const ItemData {
                    num_impls += 1;
                    continue;
                }
            }

            try!(self.path.reserve(1));
            self.path.push(try!(trait_item.name.as_ref().unwrap().maybe_clone()));
            try!(self.trait_impl(&impls[i+1-num_impls..i+1]));

            try!(file.write_all(b"\
                <tr>\
                    <td>\
                        <a href=\"./\
                    "));
            try!(file.write_all(try!(path::path(&self.path)).as_ref()));
            try!(file.write_all(b"\">"));
            try!(file.write_all(trait_item.name.as_ref().unwrap().as_ref()));
            try!(file.write_all(b"\
                        </a>\
                        "));
            if num_impls > 1 {
                try!(write!(file, " ({} times)", num_impls));
            }
            try!(file.write_all(b"\
                    </td>\
                    <td>\
                    "));
            try!(markup::short(file, &trait_item.docs.parts));
            try!(file.write_all(b"\
                    </td>\
                </tr>\
                "));

            self.path.pop();

            num_impls = 1;
        }

        try!(file.write_all(b"\
                </tbody>\
            </table>\
            "));

        Ok(())
    }
}
