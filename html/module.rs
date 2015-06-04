// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};

use html::{path, markup, Formatter};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn module(&mut self, module: &Module, docs: &Document) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Module "));
        try!(self.h1(&mut file, "Module "));

        try!(markup::short(&mut file, &docs.parts));
        try!(markup::description(&mut file, &docs.parts));

        try!(self.module_modules(&mut file, module));
        try!(self.module_types(&mut file, module));
        try!(self.module_functions(&mut file, module));
        try!(self.module_constants(&mut file, module));
        try!(self.module_statics(&mut file, module));
        try!(self.module_macros(&mut file, module));

        try!(markup::remarks(&mut file, &docs.parts));
        try!(markup::examples(&mut file, &docs.parts));
        try!(markup::see_also(&mut file, &docs.parts));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }

    fn module_modules<W: Write>(&mut self, file: &mut W, module: &Module) -> Result {
        let mut sub_mods: Vec<_> = Vec::new();

        for item in &module.items {
            match item.inner {
                Item::Module(ref m) => sub_mods.push((item, m)),
                _ => { },
            }
        }

        if sub_mods.len() == 0 {
            return Ok(());
        }

        sub_mods.sort_by(|&(m1,_), &(m2,_)| m1.name.as_ref().unwrap()
                                       .cmp(m2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Sub-modules</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, module) in &sub_mods {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.module(module, &item.docs));

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

    fn module_types<W: Write>(&mut self, file: &mut W, module: &Module) -> Result {
        let mut types: Vec<_> = Vec::new();

        for item in &module.items {
            match item.inner {
                Item::Struct(_)  => types.push((item, "Struct",  "struct_hl")),
                Item::Enum(_)    => types.push((item, "Enum",    "enum_hl")),
                Item::Typedef(_) => types.push((item, "Typedef", "typedef_hl")),
                Item::Trait(_)   => types.push((item, "Trait",   "trait_hl")),
                _ => { },
            }
        }

        if types.len() == 0 {
            return Ok(());
        }

        types.sort_by(|&(m1, _, _), &(m2, _, _)| m1.name.as_ref().unwrap()
                                            .cmp(m2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Types</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Kind</th>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, kind, class) in &types {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.type_(&item));

            try!(file.write_all(b"\
                <tr>\
                    <td class=\"\
                    "));
            try!(file.write_all(class.as_bytes()));
            try!(file.write_all(b"\">"));
            try!(file.write_all(kind.as_bytes()));
            try!(file.write_all(b"\
                    </td>
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

    fn module_functions<W: Write>(&mut self, file: &mut W, module: &Module) -> Result {
        let mut functions: Vec<_> = Vec::new();

        for item in &module.items {
            match item.inner {
                Item::Func(ref f) => functions.push((item, f)),
                _ => { },
            }
        }

        if functions.len() == 0 {
            return Ok(());
        }

        functions.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap()
                                          .cmp(f2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Functions</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, func) in &functions {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.function(item, func));

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

    fn module_constants<W: Write>(&mut self, file: &mut W, module: &Module) -> Result {
        let mut constants: Vec<_> = Vec::new();

        for item in &module.items {
            match item.inner {
                Item::Constant(ref c) => constants.push((item, c)),
                _ => { },
            }
        }

        if constants.len() == 0 {
            return Ok(());
        }

        constants.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap()
                                          .cmp(f2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Constants</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, constant) in &constants {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.constant(item, constant));

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

    fn module_statics<W: Write>(&mut self, file: &mut W, module: &Module) -> Result {
        let mut statics: Vec<_> = Vec::new();

        for item in &module.items {
            match item.inner {
                Item::Static(ref s) => statics.push((item, s)),
                _ => { },
            }
        }

        if statics.len() == 0 {
            return Ok(());
        }

        statics.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap()
                                        .cmp(f2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Statics</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, static_) in &statics {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.static_(item, static_));

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

    fn module_macros<W: Write>(&mut self, file: &mut W, module: &Module) -> Result {
        if self.path.len() != 1 {
            return Ok(());
        }

        let mut macros: Vec<_> = Vec::new();

        for item in &module.items {
            if let Item::Macro(ref m) = item.inner {
                macros.push((item, m));
            }
        }

        if macros.len() == 0 {
            return Ok(());
        }

        macros.sort_by(|&(f1, _), &(f2, _)| f1.name.as_ref().unwrap()
                                       .cmp(f2.name.as_ref().unwrap()));

        try!(file.write_all(b"\
            <h2>Macros</h2>\
            <table>\
                <thead>\
                    <tr>\
                        <th>Name</th>\
                        <th>Description</th>\
                    </tr>\
                </thead>\
                <tbody>\
                    "));

        for &(item, macro_) in &macros {
            try!(self.path.reserve(1));
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
            try!(self.macro_(item, macro_));

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
