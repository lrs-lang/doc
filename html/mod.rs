// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};
use lrs::file::{self, File};
use lrs::{process};
use lrs::string::{ByteString, SByteString};
use lrs::vec::{SVec};
use lrs::iter::{IteratorExt};

use tree::*;
use markup::{Document};

mod markup;

pub fn create(krate: Crate) -> Result {
    let docs = &krate.item.docs;
    let module = match krate.item.inner {
        Item::Module(ref m) => m,
        _ => errexit!("Crate item is not a module"),
    };

    let mut parts = try!(Vec::with_capacity(1));
    parts.push(ByteString::from_vec(try!(b"lrs".to_owned())));

    let _ = file::create_dir("doc", file::Mode::new_directory());
    let mut flags = file::Flags::new();
    flags.set_only_directory(true);
    flags.set_path_fd(true);
    let dir = try!(File::open("doc", flags));

    let mut formatter = Formatter { 
        path: parts,
        dir: dir,
    };

    formatter.module(module, docs)
}

mod path {
    #[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
    use lrs::string::{ByteString, SByteString};

    pub fn path(parts: &[SByteString]) -> Result<SByteString> {
        let mut buf = try!(title(parts)).unwrap();
        try!(buf.push_all(b".html"));
        Ok(ByteString::from_vec(buf))
    }

    pub fn title(parts: &[SByteString]) -> Result<SByteString> {
        if parts.len() == 0 {
            return Ok(ByteString::new());
        }
        let mut buf = Vec::new();
        try!(buf.push_all(parts[0].as_ref()));
        for part in &parts[1..] {
            try!(buf.push_all(b"::"));
            try!(buf.push_all(part.as_ref()));
        }
        Ok(ByteString::from_vec(buf))
    }
}


struct Formatter {
    path: SVec<SByteString>,
    dir: File,
}

impl Formatter {
    fn file(&self) -> Result<File> {
        let mut flags = file::Flags::new();
        flags.set_writable(true);
        flags.set_truncate(true);
        flags.enable_create(file::Mode::new_file());
        self.dir.rel_open(&try!(path::path(&self.path)), flags)
    }

    fn head(&self, file: &mut File, prefix: &str) -> Result {
        try!(file.write_all(b"\
            <html>\
                <head>\
                    <link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\" />\
                    <title>\
            "));
        try!(file.write_all(prefix.as_bytes()));
        try!(file.write_all(try!(path::title(&self.path)).as_ref()));
        try!(file.write_all(b"\
                    </title>\
                </head>\
                <body>\
            "));
        Ok(())
    }

    fn foot(&self, file: &mut File) -> Result {
        try!(file.write_all(b"\
                </body>\
            </html>\
            "));
        Ok(())
    }

    fn h1(&self, file: &mut File, prefix: &str) -> Result {
        try!(file.write_all(b"<h1>"));
        try!(file.write_all(prefix.as_bytes()));

        let mut first = true;
        for i in 1..self.path.len()+1 {
            if first {
                first = false;
            } else {
                try!(file.write_all(b"::"));
            }
            try!(file.write_all(b"<a href=\"./"));
            try!(file.write_all(try!(path::path(&self.path[..i])).as_ref()));
            try!(file.write_all(b"\">"));
            try!(file.write_all(self.path[i-1].as_ref()));
            try!(file.write_all(b"</a>"));
        }

        try!(file.write_all(b"</h1>"));
        Ok(())
    }

    fn module(&mut self, module: &Module, docs: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Module "));
        try!(self.h1(&mut file, "Module "));

        try!(markup::short(&mut file, &docs.parts));
        try!(markup::description(&mut file, &docs.parts));

        try!(self.sub_modules(&mut file, module));
        try!(self.types(&mut file, module));

        try!(self.foot(&mut file));
        Ok(())
    }

    fn sub_modules(&mut self, file: &mut File, module: &Module) -> Result {
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

        sub_mods.sort_by(|&(m1,_), &(m2,_)| m1.name.as_ref().unwrap().as_ref()
                                    .cmp(m2.name.as_ref().unwrap().as_ref()));

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

    fn types(&mut self, file: &mut File, module: &Module) -> Result {
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

        types.sort_by(|&(m1, _, _), &(m2, _, _)| m1.name.as_ref().unwrap().as_ref()
                                            .cmp(m2.name.as_ref().unwrap().as_ref()));

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
            try!(self.type_(&item, &item.docs));

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

    fn type_(&mut self, item: &ItemData, docs: &Document) -> Result {
        match item.inner {
            Item::Struct(ref  s) => self.struct_(item, s, docs),
            Item::Enum(ref    e) => self.enum_(e,   docs),
            Item::Typedef(ref t) => self.typedef(t, docs),
            Item::Trait(ref   t) => self.trait_(t,  docs),
            _ => abort!(),
        }
    }

    fn struct_(&mut self, item: &ItemData, strukt: &Struct, docs: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Struct "));
        try!(self.h1(&mut file, "Struct "));

        try!(markup::short(&mut file, &docs.parts));

        try!(self.struct_syntax(&mut file, strukt));
        try!(self.struct_fields(&mut file, strukt, docs));
        try!(self.static_methods(&mut file, item));

        try!(markup::remarks(&mut file, &docs.parts));
        try!(markup::see_also(&mut file, &docs.parts));

        Ok(())
    }

    fn struct_syntax(&mut self, file: &mut File, strukt: &Struct) -> Result {
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

    fn struct_fields(&mut self, mut file: &mut File, strukt: &Struct,
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

    fn static_methods(&mut self, mut file: &mut File, item: &ItemData) -> Result {
        let impls = item.impls.borrow();

        let mut methods: Vec<_> = Vec::new();

        for impl_ in &*impls {
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

        if methods.len() == 0 {
            return Ok(());
        }

        methods.sort_by(|&(_, i1, _), &(_, i2, _)| i1.name.as_ref().unwrap().as_ref()
                                              .cmp(i2.name.as_ref().unwrap().as_ref()));

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
            self.path.push(try!(item.name.as_ref().unwrap().clone()));
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

    fn method(&mut self, impl_: &Impl, item: &ItemData, method: &Method) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Method "));
        try!(self.h1(&mut file, "Method "));

        try!(markup::short(&mut file, &item.docs.parts));

        try!(self.method_syntax(&mut file, impl_, item, method));

        Ok(())
    }

    fn method_syntax(&mut self, file: &mut File, impl_: &Impl, item: &ItemData,
                     method: &Method) -> Result {
        try!(file.write_all(b"\
            <h2>Syntax</h2>\
            <pre>\
                impl\
            "));

        // impl block

        let mut have_where_predicates = impl_.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &impl_.generics));

        try!(file.write_all(b" "));
        try!(write_raw_type(file, &impl_.for_));

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &impl_.generics, "    "));
            try!(file.write_all(b"\n{\n"));
        } else {
            try!(file.write_all(b" {\n"));
        }

        // fn block

        try!(file.write_all(b"    "));
        if method.unsaf {
            try!(file.write_all(b"unsafe "));
        }
        try!(file.write_all(b"fn "));
        try!(file.write_all(self.path.last().as_ref().unwrap().as_ref()));

        have_where_predicates = method.generics.where_predicates.len() > 0;
        have_where_predicates |= try!(angle_generics(file, &method.generics));

        try!(file.write_all(b"("));

        let mut first = true;
        for arg in &method.decl.inputs {
            if !first {
                try!(file.write_all(b", "));
            }
            first = false;
            try!(file.write_all(arg.name.as_ref()));
            try!(file.write_all(b": "));
            try!(write_raw_type(file, &arg.type_));
        }

        try!(file.write_all(b")"));

        match method.decl.output {
            FuncRetTy::NoReturn => {
                try!(file.write_all(b" -> !"));
            },
            FuncRetTy::Return(ref t) => {
                try!(file.write_all(b" -> "));
                try!(write_raw_type(file, t));
            },
            FuncRetTy::Unit => { },
        }

        if have_where_predicates {
            try!(file.write_all(b"\n"));
            try!(where_predicates(file, &method.generics, "        "));
        }

        try!(file.write_all(b"\
                \n}\
            </pre>\
            "));

        Ok(())
    }

    fn enum_(&mut self, _: &Enum, _: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Enum "));
        try!(self.h1(&mut file, "Enum "));
        Ok(())
    }

    fn typedef(&mut self, _: &Typedef, _: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Typedef "));
        try!(self.h1(&mut file, "Typedef "));
        Ok(())
    }

    fn trait_(&mut self, _: &Trait, _: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Trait "));
        try!(self.h1(&mut file, "Trait "));
        Ok(())
    }
}

fn write_ty_param_bounds(file: &mut File, bounds: &[TyParamBound]) -> Result {
    let mut first = true;
    for bound in bounds {
        if !first {
            try!(file.write_all(b" + "));
        }
        first = false;
        try!(write_ty_param_bound(file, bound));
    }
    Ok(())
}

fn write_ty_param_bound(file: &mut File, bound: &TyParamBound) -> Result {
    match *bound {
        TyParamBound::Lifetime(ref l) => {
            try!(file.write_all(l.as_ref()));
        },
        TyParamBound::Trait(ref t) => {
            if t.maybe {
                try!(file.write_all(b"?"));
            }
            try!(write_raw_type(file, &t.trait_.trait_));
            for lt in &t.trait_.lifetimes {
                try!(file.write_all(b"+"));
                try!(file.write_all(lt.as_ref()));
            }
        },
    }
    Ok(())
}

fn write_angle_params(file: &mut File, lts: &[SByteString], types: &[Type],
                      bindings: &[TypeBinding]) -> Result {
    let mut first = true;
    for lt in lts {
        if !first {
            try!(file.write_all(b", "));
        }
        first = false;
        try!(file.write_all(lt.as_ref()));
    }
    for t in types {
        if !first {
            try!(file.write_all(b", "));
        }
        first = false;
        try!(write_raw_type(file, t));
    }
    for b in bindings {
        if !first {
            try!(file.write_all(b", "));
        }
        first = false;
        try!(file.write_all(b.name.as_ref()));
        try!(file.write_all(b" = "));
        try!(write_raw_type(file, &b.ty));
    }
    Ok(())
}

fn write_raw_type(file: &mut File, t: &Type) -> Result {
    match *t {
        Type::ResolvedPath(ref p) => {
            let mut first = !p.path.global;
            for (i, segment) in p.path.segments.iter().enumerate() {
                if !first {
                    try!(file.write_all(b"::"));
                }
                first = false;
                if i == p.path.segments.len() - 1 {
                    if let Some(ref item) = *p.item.borrow() {
                        try!(file.write_all(b"<a href=\"./"));
                        try!(write_full_path(file, item));
                        try!(file.write_all(b".html\">"));
                    }
                    try!(file.write_all(segment.name.as_ref()));
                    if p.item.borrow().is_some() {
                        try!(file.write_all(b"</a>"));
                    }
                } else {
                    try!(file.write_all(segment.name.as_ref()));
                }
                match segment.params {
                    PathParameters::AngleBracketed(ref a) => {
                        if a.lifetimes.len() + a.ty_params.len() + a.bindings.len() > 0 {
                            try!(file.write_all(b"&lt;"));
                            try!(write_angle_params(file, &a.lifetimes, &a.ty_params,
                                                    &a.bindings));
                            try!(file.write_all(b"&gt;"));
                        }
                    }
                    PathParameters::Parenthesized(ref p) => {
                        try!(file.write_all(b"("));
                        try!(write_angle_params(file, &[], &p.args, &[]));
                        try!(file.write_all(b")"));
                        if let Some(ref rv) = p.return_value {
                            try!(file.write_all(b" -> "));
                            try!(write_raw_type(file, rv));
                        }
                    }
                }
            }
        },
        Type::Generic(ref g) => {
            try!(file.write_all(g.name.as_ref()));
        },
        Type::Primitive(p) => {
            let res = match p {
                Primitive::Isize      => file.write_all(b"isize"),
                Primitive::I8         => file.write_all(b"i8"),
                Primitive::I16        => file.write_all(b"i16"),
                Primitive::I32        => file.write_all(b"i32"),
                Primitive::I64        => file.write_all(b"i64"),
                Primitive::Usize      => file.write_all(b"usize"),
                Primitive::U8         => file.write_all(b"u8"),
                Primitive::U16        => file.write_all(b"u16"),
                Primitive::U32        => file.write_all(b"u32"),
                Primitive::U64        => file.write_all(b"u64"),
                Primitive::F32        => file.write_all(b"f32"),
                Primitive::F64        => file.write_all(b"f64"),
                Primitive::Char       => file.write_all(b"char"),
                Primitive::Bool       => file.write_all(b"bool"),
                Primitive::Str        => file.write_all(b"str"),
                Primitive::Slice      => file.write_all(b"[..]"),
                Primitive::Array      => file.write_all(b"[..;..]"),
                Primitive::Tuple      => file.write_all(b"(..)"),
                Primitive::RawPointer => file.write_all(b"*"),
            };
            try!(res);
        },
        Type::BareFunction(_) => {
        },
        Type::Tuple(ref ts) => {
            try!(file.write_all(b"("));
            if ts.fields.len() == 1 {
                try!(write_raw_type(file, &ts.fields[0]));
                try!(file.write_all(b",)"));
            } else {
                let mut first = true;
                for t in &ts.fields {
                    if !first {
                        try!(file.write_all(b", "));
                    }
                    first = false;
                    try!(write_raw_type(file, t));
                }
                try!(file.write_all(b")"));
            }
        },
        Type::Slice(ref s) => {
            try!(file.write_all(b"["));
            try!(write_raw_type(file, &s.ty));
            try!(file.write_all(b"]"));
        },
        Type::Array(ref a) => {
            try!(file.write_all(b"["));
            try!(write_raw_type(file, &a.ty));
            try!(file.write_all(b"; "));
            try!(file.write_all(a.initializer.as_ref()));
            try!(file.write_all(b"]"));
        },
        Type::Bottom => {
        },
        Type::Pointer(ref p) => {
            try!(file.write_all(b"*"));
            if p.mutable {
                try!(file.write_all(b"mut "));
            } else {
                try!(file.write_all(b"const "));
            }
            try!(write_raw_type(file, &p.ty));
        },
        Type::Ref(ref r) => {
            try!(file.write_all(b"&amp;"));
            if let Some(ref lt) = r.lifetime {
                try!(file.write_all(lt.as_ref()));
                try!(file.write_all(b" "));
            }
            if r.mutable {
                try!(file.write_all(b"mut "));
            }
            try!(write_raw_type(file, &r.ty));
        },
        Type::UfcsPath(ref u) => {
            try!(file.write_all(b"&lt;"));
            try!(write_raw_type(file, &u.self_ty));
            try!(file.write_all(b" as "));
            try!(write_raw_type(file, &u.trait_));
            try!(file.write_all(b"&gt;::"));
            try!(file.write_all(u.target.as_ref()));
        },
        Type::Infer => {
            try!(file.write_all(b"_"));
        },
        Type::HkltBound(_) => {
        },
    }
    Ok(())
}

fn write_full_path<W: Write>(dst: &mut W, dstitem: &ItemData) -> Result {
    if let Some(ref parent) = *dstitem.parent.borrow() {
        try!(write_full_path(dst, parent));
        try!(dst.write_all(b"::"));
    } else {
        try!(dst.write_all(b"lrs"));
    }
    if let Some(ref name) = dstitem.name {
        try!(dst.write_all(name.as_ref()));
    }
    Ok(())
}

fn angle_generics(file: &mut File,  generics: &Generics) -> Result<bool> {
    if generics.lifetimes.len() + generics.type_params.len() == 0 {
        return Ok(false);
    }

    let mut have_where_predicates = false;

    try!(file.write_all(b"&lt;"));
    let mut first = true;
    for lt in &generics.lifetimes {
        if !first {
            try!(file.write_all(b", "));
        }
        first = false;
        try!(file.write_all(lt.as_ref()));
    }
    for t in &generics.type_params {
        if !first {
            try!(file.write_all(b", "));
        }
        first = false;
        try!(file.write_all(t.name.as_ref()));
        if let Some(ref t) = t.default {
            try!(file.write_all(b" = "));
            try!(write_raw_type(file, t));
        }
        have_where_predicates |= t.bounds.len() > 0;
    }
    try!(file.write_all(b"&gt;"));

    Ok(have_where_predicates)
}

fn where_predicates(file: &mut File, generics: &Generics, prefix: &str) -> Result {
    let mut first = true;
    for t in &generics.type_params {
        if t.bounds.len() == 0 {
            continue;
        }
        try!(file.write_all(prefix.as_bytes()));
        if first {
            try!(file.write_all(b"where "));
        } else {
            try!(file.write_all(b"\n     "));
        }
        first = false;
        try!(write_ty_param_bounds(file, &t.bounds));
        try!(file.write_all(b","));
    }
    for t in &generics.where_predicates {
        try!(file.write_all(prefix.as_bytes()));
        if first {
            try!(file.write_all(b"where "));
        } else {
            try!(file.write_all(b"\n      "));
        }
        first = false;
        match *t {
            WherePredicate::Bound(ref b) => {
                try!(write_raw_type(file, &b.ty));
                try!(file.write_all(b": "));
                try!(write_ty_param_bounds(file, &b.bounds));
            },
            WherePredicate::Region(ref r) => {
                try!(file.write_all(r.lt.as_ref()));
                try!(file.write_all(b": "));
                let mut first = true;
                for lt in &r.bounds {
                    if !first {
                        try!(file.write_all(b"+"));
                    }
                    first = false;
                    try!(file.write_all(lt.as_ref()));
                }
            },
            WherePredicate::Eq(ref e) => {
                try!(write_raw_type(file, &e.lhs));
                try!(file.write_all(b" = "));
                try!(write_raw_type(file, &e.rhs));
            },
        }
        try!(file.write_all(b","));
    }
    Ok(())
}
