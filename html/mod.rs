// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Beware of the shit code

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};
use lrs::file::{self, File};
use lrs::string::{ByteString, SByteString};
use lrs::vec::{SVec};
use lrs::iter::{IteratorExt};

use tree::*;

mod markup;

mod trait_;
mod trait_method;
mod typedef;
mod enum_;
mod variant;
mod constant;
mod static_;
mod function;
mod struct_;
mod type_;
mod module;
mod method;

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
}

fn write_ty_param_bounds<W: Write>(file: &mut W, bounds: &[TyParamBound]) -> Result {
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

fn write_ty_param_bound<W: Write>(file: &mut W, bound: &TyParamBound) -> Result {
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

fn write_angle_params<W: Write>(file: &mut W, lts: &[SByteString], types: &[Type],
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

fn write_raw_type<W: Write>(file: &mut W, t: &Type) -> Result {
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
            try!(file.write_all(b"!"));
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

fn fn_in_out<W: Write>(dst: &mut W, slf: &SelfTy, decl: &FnDecl) -> Result {
    try!(dst.write_all(b"("));

    let have_slf = match *slf {
        SelfTy::Static => false,
        SelfTy::Value => {
            try!(dst.write_all(b"self"));
            true
        },
        SelfTy::Borrowed(ref lt, mutable) => {
            try!(dst.write_all(b"&"));
            if let Some(ref s) = *lt {
                try!(dst.write_all(s.as_ref()));
                try!(dst.write_all(b" "));
            }
            if mutable {
                try!(dst.write_all(b"mut "));
            }
            try!(dst.write_all(b"self"));
            true
        },
        SelfTy::Explicit(ref t) => {
            try!(dst.write(b"self: "));
            try!(write_raw_type(dst, t));
            true
        },
    };

    let mut first = !have_slf;
    for arg in &decl.inputs {
        if !first {
            try!(dst.write_all(b", "));
        }
        first = false;
        try!(dst.write_all(arg.name.as_ref()));
        try!(dst.write_all(b": "));
        try!(write_raw_type(dst, &arg.type_));
    }

    try!(dst.write_all(b")"));

    match decl.output {
        FuncRetTy::NoReturn => {
            try!(dst.write_all(b" -> !"));
        },
        FuncRetTy::Return(ref t) => {
            try!(dst.write_all(b" -> "));
            try!(write_raw_type(dst, t));
        },
        FuncRetTy::Unit => { },
    }

    Ok(())
}

fn write_abi<W: Write>(dst: &mut W, abi: &Abi) -> Result<bool> {
    if let Abi::Rust = *abi {
        return Ok(false);
    }
    match *abi {
        Abi::Rust => 0,
        Abi::C => try!(dst.write_all(b"extern")),
        Abi::System => try!(dst.write_all(b"extern \"system\"")),
        Abi::RustIntrinsic => try!(dst.write_all(b"extern \"rust-intrinsic\"")),
        Abi::RustCall => try!(dst.write_all(b"extern \"rust-call\"")),
    };
    Ok(true)
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

fn angle_generics<W: Write>(file: &mut W,  generics: &Generics) -> Result<bool> {
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

fn where_predicates<W: Write>(file: &mut W, generics: &Generics, prefix: &str) -> Result {
    let mut first = true;
    for t in &generics.type_params {
        if t.bounds.len() == 0 {
            continue;
        }
        if first {
            try!(file.write_all(prefix.as_bytes()));
            try!(file.write_all(b"where "));
        } else {
            try!(file.write_all(b"\n      "));
            try!(file.write_all(prefix.as_bytes()));
        }
        first = false;
        try!(write_ty_param_bounds(file, &t.bounds));
        try!(file.write_all(b","));
    }
    for t in &generics.where_predicates {
        if first {
            try!(file.write_all(prefix.as_bytes()));
            try!(file.write_all(b"where "));
        } else {
            try!(file.write_all(b"\n      "));
            try!(file.write_all(prefix.as_bytes()));
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
