// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::error::{self, InvalidArgument};
use lrs::vec::{SVec};
use lrs::string::{SByteString, AsByteStr};
use lrs::bx::{Box};
use lrs::rc::{Arc};

use json::{Slice, Value};
use ast::*;
use hashmap::{ItemMap};

macro_rules! error {
    ($fmt:expr) => { error!(concat!($fmt, "{}"), "") };
    ($fmt:expr, $($arg:tt)*) => {{
        errln!(concat!("lrs_doc: Error: ", $fmt), $($arg)*);
        return Err(error::InvalidArgument);
    }};
}

macro_rules! warning {
    ($fmt:expr) => { warning!(concat!($fmt, "{}"), "") };
    ($fmt:expr, $($arg:tt)*) => {{
        errln!(concat!("lrs_doc: Warning: ", $fmt), $($arg)*);
    }};
}

macro_rules! box_try {
    ($e:expr) => {
        match $e {
            Ok(e) => e,
            Err((_, e)) => return Err(e),
        }
    }
}

pub const SCHEMA: &'static [u8] = b"0.8.3";

pub fn parse(json: &Value) -> Result<Crate> {
    let mut map = ItemMap::new();

    let mut fields = [("schema", None), ("crate", None)];
    try!(collect_object(json, &mut fields, "input"));

    let schema = try!(collect_string(fields[0].1.unwrap(), "input", "schema"));
    if schema != SCHEMA {
        warning!("Unexpected schema. Expected {:?} found {:?}", SCHEMA.as_byte_str(),
                 schema);
    }

    krate(fields[1].1.unwrap(), &mut map)
}

fn krate(json: &Value, map: &mut ItemMap) -> Result<Crate> {
    let mut fields = [("module", None)];
    try!(collect_object(json, &mut fields, "crate"));

    let module = try!(item_data(fields[0].1.unwrap(), map));
    Ok(Crate { item: module })
}

fn item_datas(json: &Value, map: &mut ItemMap) -> Result<SVec<Arc<ItemData>>> {
    let array = try!(collect_array(json, "?", "items"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(item_data(l, map)));
    }
    Ok(vec)
}

fn item_data(json: &Value, map: &mut ItemMap) -> Result<Arc<ItemData>> {
    let mut fields = [("name", None), ("attrs", None), ("inner", None),
                      ("visibility", None), ("def_id", None)];
    try!(collect_object(json, &mut fields, "item"));

    let name = match *fields[0].1.unwrap() {
        Value::String(ref s) => {
            Some(s.clone().unwrap())
        },
        _ => None,
    };
    let attrs  = try!(attributes(fields[1].1.unwrap()));
    let inner  = try!(item(fields[2].1.unwrap(), map));
    let public = match *fields[3].1.unwrap() {
        Value::Null => true,
        _ => try!(visibility(fields[3].1.unwrap())),
    };
    let node   = try!(def_id(fields[4].1.unwrap()));

    let item = Arc::new(ItemData {
        name: name,
        attrs: attrs,
        inner: inner,
        public: public,
        node: node,
    }).unwrap();

    map.add(node, item.new_ref());

    Ok(item)
}

fn attributes(json: &Value) -> Result<SVec<Attribute>> {
    let array = try!(collect_array(json, "?", "attributes"));
    let mut attributes = try!(Vec::with_capacity(array.len()));
    for val in array {
        attributes.push(try!(attribute(val)));
    }
    Ok(attributes)
}

fn attribute(json: &Value) -> Result<Attribute> {
    let (variant, fields) = try!(collect_enum(json, "Attribute"));

    match variant.as_ref() {
        b"Word" => {
            if fields.len() != 1 { error!("word attribute has unexpected structure") }
            let string = try!(collect_string(&fields[0], "word attribute", "fields[0]"));
            let string = try!(string.clone());
            Ok(Attribute::Word(string))
        },
        b"List" => {
            if fields.len() != 2 { error!("list attribute has unexpected structure") }
            let one = try!(collect_string(&fields[0], "list attribute", "fields[0]"));
            let two = try!(attributes(&fields[1]));
            let one = try!(one.clone());
            Ok(Attribute::List(one, two))
        },
        b"NameValue" => {
            if fields.len() != 2 { error!("list attribute has unexpected structure") }
            let one = try!(collect_string(&fields[0], "namevalue attribute", "fields[0]"));
            let two = try!(collect_string(&fields[1], "namevalue attribute", "fields[1]"));
            let one = try!(one.clone());
            let two = try!(two.clone());
            Ok(Attribute::NameValue(one, two))
        },
        _ => error!("unexpected attribute variant {:?}", variant),
    }
}

fn visibility(json: &Value) -> Result<bool> {
    let s = try!(collect_string(json, "?", "visibility"));
    match s.as_ref() {
        b"Public" => Ok(true),
        b"Inherited" => Ok(false),
        _ => error!("visibility contains unexpected value: {:?}", s),
    }
}

fn def_id(json: &Value) -> Result<u64> {
    let mut fields = [("node", None)];
    try!(collect_object(json, &mut fields, "def_id"));
    let n = try!(collect_int(fields[0].1.unwrap(), "def_id", "node"));
    Ok(n as u64)
}

fn item(json: &Value, map: &mut ItemMap) -> Result<Item> {
    let (variant, fields) = try!(collect_enum(json, "ItemEnum"));

    match variant.as_ref() {
        b"ImportItem"          => item_import(fields),
        b"StructItem"          => item_struct(fields, map),
        b"EnumItem"            => item_enum(fields, map),
        b"FunctionItem"        => item_func(fields),
        b"ModuleItem"          => item_module(fields, map),
        b"TypedefItem"         => item_typedef(fields),
        b"StaticItem"          => item_static(fields),
        b"ConstantItem"        => item_constant(fields),
        b"TraitItem"           => item_trait(fields, map),
        b"ImplItem"            => item_impl(fields, map),
        b"TyMethodItem"        => item_method_decl(fields),
        b"MethodItem"          => item_method(fields),
        b"StructFieldItem"     => item_struct_field(fields),
        b"VariantItem"         => item_variant(fields, map),
        b"ForeignFunctionItem" => item_extern_func(fields),
        b"ForeignStaticItem"   => item_extern_static(fields),
        b"MacroItem"           => item_macro(fields),
        b"PrimitiveItem"       => item_primitive(fields),
        b"AssociatedTypeItem"  => item_assoc_type(fields),
        b"DefaultImplItem"     => item_default_impl(fields),
        _ => error!("Unexpected item: {:?}", variant),
    }
}

fn item_import(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("import item with {} fields", fields.len()); }
    let (variant, fields) = try!(collect_enum(&fields[0], "Import"));
    if variant != "GlobImport" { error!("unexpected import variant: {:?}", variant); }
    if fields.len() != 1 { error!("glob import with {} fields", fields.len()); }
    let source = try!(glob_import(&fields[0]));
    Ok(Item::GlobImport(source))
}

fn glob_import(json: &Value) -> Result<GlobImport> {
    let mut fields = [("path", None), ("did", None)];
    try!(collect_object(json, &mut fields, "glob import"));

    let path = try!(path(fields[0].1.unwrap()));
    let node = match *fields[1].1.unwrap() {
        Value::Object(_) => Some(try!(def_id(fields[1].1.unwrap()))),
        _ => None,
    };

    Ok(GlobImport { path: path, node: node })
}

fn path(json: &Value) -> Result<Path> {
    let mut fields = [("global", None), ("segments", None)];
    try!(collect_object(json, &mut fields, "path"));

    let global = try!(collect_bool(fields[0].1.unwrap(), "path", "global"));
    let segments = try!(path_segments(fields[1].1.unwrap()));

    Ok(Path { global: global, segments: segments })
}

fn path_segments(json: &Value) -> Result<SVec<PathSegment>> {
    let array = try!(collect_array(json, "path", "segments"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for seg in array {
        vec.push(try!(path_segment(seg)));
    }
    Ok(vec)
}

fn path_segment(json: &Value) -> Result<PathSegment> {
    let mut fields = [("name", None), ("params", None)];
    try!(collect_object(json, &mut fields, "path_segment"));

    let name = try!(collect_string(fields[0].1.unwrap(), "path_segment", "name"));
    let params = try!(path_params(fields[1].1.unwrap()));
    let name = try!(name.clone());

    Ok(PathSegment { name: name, params: params })
}

fn path_params(params: &Value) -> Result<PathParameters> {
    let (variant, fields) = try!(collect_enum(params, "PathParameters"));
    match variant.as_ref() {
        b"AngleBracketed" => {
            if fields.len() != 3 { error!("angle bracketed path params with {} fields",
                                           fields.len()); }
            let lifetimes = try!(lifetimes(&fields[0]));
            let types = try!(types(&fields[1]));
            let bindings = try!(type_bindings(&fields[2]));
            Ok(PathParameters::AngleBracketed(lifetimes, types, bindings))
        },
        b"Parenthesized" => {
            if fields.len() != 2 { error!("paranthesized path parmas with {} fields",
                                          fields.len()); }
            let inputs = try!(types(&fields[0]));
            let output = match fields[1] {
                Value::Null => None,
                _ => Some(try!(type_(&fields[1]))),
            };
            Ok(PathParameters::Parenthesized(inputs, output))
        },
        _ => {
            error!("unexpected path_params variant: {:?}", variant);
        },
    }
}

fn lifetimes(json: &Value) -> Result<SVec<SByteString>> {
    let array = try!(collect_array(json, "?", "lifetimes"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(lifetime(l)));
    }
    Ok(vec)
}

fn lifetime(json: &Value) -> Result<SByteString> {
    let mut fields = [("_field0", None)];
    try!(collect_object(json, &mut fields, "Lifetime"));
    let s = try!(collect_string(fields[0].1.unwrap(), "Lifetime", "_field0"));
    let s = try!(s.clone());
    Ok(s)
}

fn types(json: &Value) -> Result<SVec<Type>> {
    let array = try!(collect_array(json, "?", "types"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(type_(l)));
    }
    Ok(vec)
}

fn type_(json: &Value) -> Result<Type> {
    let (variant, fields) = try!(collect_enum(json, "type"));
    match variant.as_ref() {
        b"ResolvedPath" => type_resolved_path(fields),
        b"Generic"      => type_generic(fields),
        b"Primitive"    => type_primitive(fields),
        b"BareFunction" => type_bare_function(fields),
        b"Tuple"        => type_tuple(fields),
        b"Vector"       => type_slice(fields),
        b"FixedVector"  => type_array(fields),
        b"Bottom"       => type_bottom(fields),
        b"RawPointer"   => type_pointer(fields),
        b"BorrowedRef"  => type_ref(fields),
        b"QPath"        => type_ufcs_path(fields),
        b"Infer"        => type_infer(fields),
        b"PolyTraitRef" => type_hklt_bound(fields),
        _ => error!("Unexpected type: {:?}", variant),
    }
}

fn type_resolved_path(fields: &Slice) -> Result<Type> {
    if fields.len() != 3 { error!("resolved path type with {} fields", fields.len()); }
    let path = try!(path(&fields[0]));
    let typarams = match fields[1] {
        Value::Null => None,
        _ => Some(try!(ty_param_bounds(&fields[1]))),
    };
    let node = try!(def_id(&fields[2]));
    Ok(Type::ResolvedPath(path, typarams, node))
}

fn ty_param_bounds(json: &Value) -> Result<SVec<TyParamBound>> {
    let array = try!(collect_array(json, "?", "ty_param_bounds"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(ty_param_bound(l)));
    }
    Ok(vec)
}

fn ty_param_bound(json: &Value) -> Result<TyParamBound> {
    let (variant, fields) = try!(collect_enum(json, "ty_param_bound"));
    match variant.as_ref() {
        b"RegionBound" => {
            if fields.len() != 1 { error!("region bound with {} fields", fields.len()) }
            let lt = try!(lifetime(&fields[0]));
            Ok(TyParamBound::Lifetime(lt))
        },
        b"TraitBound" => {
            if fields.len() != 2 { error!("trait bound with {} fields", fields.len()) }
            let poly_trait = try!(poly_trait(&fields[0]));
            let maybe = try!(trait_bound_modifier(&fields[1]));
            Ok(TyParamBound::Trait(poly_trait, maybe))
        },
        _ => error!("unexpected TyParamBound variant: {:?}", variant),
    }
}

fn trait_bound_modifier(json: &Value) -> Result<bool> {
    let (variant, _) = try!(collect_enum(json, "TraitBoundModifier"));
    match variant.as_ref() {
        b"None" => Ok(false),
        b"Maybe" => Ok(true),
        _ => error!("Unexpected TraitBoundModifier variant: {:?}", variant),
    }
}

fn poly_trait(json: &Value) -> Result<PolyTrait> {
    let mut fields = [("trait_", None), ("lifetimes", None)];
    try!(collect_object(json, &mut fields, "PolyTrait"));
    let trait_ = try!(type_(fields[0].1.unwrap()));
    let lifetimes = try!(lifetimes(fields[1].1.unwrap()));
    Ok(PolyTrait { trait_: trait_, lifetimes: lifetimes })
}

fn type_generic(fields: &Slice) -> Result<Type> {
    if fields.len() != 1 { error!("generic type with {} fields", fields.len()); }
    let s = try!(collect_string(&fields[0], "generic type", "unnamed"));
    Ok(Type::Generic(try!(s.clone())))
}

fn type_primitive(fields: &Slice) -> Result<Type> {
    if fields.len() != 1 { error!("primitive type with {} fields", fields.len()); }
    let p = try!(primitive(&fields[0]));
    Ok(Type::Primitive(p))
}

fn primitive(json: &Value) -> Result<Primitive> {
    let s = try!(collect_string(json, "?", "primitive"));
    match s.as_ref() {
        b"Isize"               => Ok(Primitive::Isize),
        b"I8"                  => Ok(Primitive::I8),
        b"I16"                 => Ok(Primitive::I16),
        b"I32"                 => Ok(Primitive::I32),
        b"I64"                 => Ok(Primitive::I64),
        b"Usize"               => Ok(Primitive::Usize),
        b"U8"                  => Ok(Primitive::U8),
        b"U16"                 => Ok(Primitive::U16),
        b"U32"                 => Ok(Primitive::U32),
        b"U64"                 => Ok(Primitive::U64),
        b"F32"                 => Ok(Primitive::F32),
        b"F64"                 => Ok(Primitive::F64),
        b"Char"                => Ok(Primitive::Char),
        b"Bool"                => Ok(Primitive::Bool),
        b"Str"                 => Ok(Primitive::Str),
        b"Slice"               => Ok(Primitive::Slice),
        b"Array"               => Ok(Primitive::Array),
        b"PrimitiveTuple"      => Ok(Primitive::Tuple),
        b"PrimitiveRawPointer" => Ok(Primitive::RawPointer),
        _ => error!("unexpected primitive variant: {:?}", s),
    }
}

fn type_bare_function(fields: &Slice) -> Result<Type> {
    if fields.len() != 1 { error!("bare function type with {} fields", fields.len()); }

    let mut fields_ = [("unsafety", None), ("generics", None), ("decl", None),
                     ("abi", None)];
    try!(collect_object(&fields[0], &mut fields_, "bare function type"));
    let unsafety = try!(collect_string(fields_[0].1.unwrap(), "bare function type",
                                       "unsafety"));
    let generics = try!(generics(fields_[1].1.unwrap()));
    let decl = try!(fn_decl(fields_[2].1.unwrap()));
    let abi = try!(collect_string(fields_[3].1.unwrap(), "bare function type", "abi"));
    
    let unsaf = unsafety == "Unsafe";

    let bare_decl = BareFunctionDecl {
        unsaf: unsaf,
        generics: generics,
        decl: decl,
        abi: try!(abi.clone()),
    };

    Ok(Type::BareFunction(box_try!(Box::new(bare_decl))))
}

fn type_tuple(fields: &Slice) -> Result<Type> {
    if fields.len() != 1 { error!("tuple type with {} fields", fields.len()); }
    let array = try!(collect_array(&fields[0], "tuple type", "unnamed"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for field in array {
        vec.push(try!(type_(field)));
    }
    Ok(Type::Tuple(vec))
}

fn type_slice(fields: &Slice) -> Result<Type> {
    if fields.len() != 1 { error!("slice type with {} fields", fields.len()); }
    let ty = try!(type_(&fields[0]));
    Ok(Type::Slice(box_try!(Box::new(ty))))
}

fn type_array(fields: &Slice) -> Result<Type> {
    if fields.len() != 2 { error!("array type with {} fields", fields.len()); }
    let ty = try!(type_(&fields[0]));
    let len = try!(collect_string(&fields[1], "array type", "unnamed"));
    Ok(Type::Array(box_try!(Box::new(ty)), try!(len.clone())))
}

fn type_bottom(fields: &Slice) -> Result<Type> {
    if fields.len() != 0 { error!("bottom type with {} fields", fields.len()); }
    Ok(Type::Bottom)
}

fn type_pointer(fields: &Slice) -> Result<Type> {
    if fields.len() != 2 { error!("bottom type with {} fields", fields.len()); }
    let mutable = try!(mutability(&fields[0]));
    let ty = try!(type_(&fields[1]));
    Ok(Type::Pointer(mutable, box_try!(Box::new(ty))))
}

fn type_ref(fields: &Slice) -> Result<Type> {
    if fields.len() != 3 { error!("ref type with {} fields", fields.len()); }
    let lifetime = match fields[0] {
        Value::Null => None,
        _ => Some(try!(lifetime(&fields[0]))),
    };
    let mutable = try!(mutability(&fields[1]));
    let ty = try!(type_(&fields[2]));
    Ok(Type::Ref(lifetime, mutable, box_try!(Box::new(ty))))
}

fn type_ufcs_path(fields: &Slice) -> Result<Type> {
    if fields.len() != 3 { error!("ufcs type with {} fields", fields.len()); }
    let name = try!(collect_string(&fields[0], "ufcs type", "name"));
    let self_ty = try!(type_(&fields[1]));
    let trait_ty = try!(type_(&fields[2]));
    let name = try!(name.clone());
    Ok(Type::UfcsPath(name, box_try!(Box::new(self_ty)), box_try!(Box::new(trait_ty))))
}

fn type_infer(fields: &Slice) -> Result<Type> {
    if fields.len() != 0 { error!("infer type with {} fields", fields.len()); }
    Ok(Type::Infer)
}

fn type_hklt_bound(fields: &Slice) -> Result<Type> {
    if fields.len() != 1 { error!("hklt bound type with {} fields", fields.len()); }
    let bounds = try!(ty_param_bounds(&fields[0]));
    Ok(Type::HkltBound(bounds))
}

fn type_bindings(json: &Value) -> Result<SVec<TypeBinding>> {
    let array = try!(collect_array(json, "?", "type_bindings"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(type_binding(l)));
    }
    Ok(vec)
}

fn type_binding(json: &Value) -> Result<TypeBinding> {
    let mut fields = [("name", None), ("ty", None)];
    try!(collect_object(json, &mut fields, "TypeBinding"));
    let name = try!(collect_string(fields[0].1.unwrap(), "type_bindings", "name"));
    let ty = try!(type_(fields[1].1.unwrap()));
    let name = try!(name.clone());
    Ok(TypeBinding { name: name, ty: ty })
}

fn item_struct(fields: &Slice, map: &mut ItemMap) -> Result<Item> {
    if fields.len() != 1 { error!("struct item with {} fields", fields.len()); }
    let struct_ = try!(struct_(&fields[0], map));
    Ok(Item::Struct(struct_))
}

fn struct_(json: &Value, map: &mut ItemMap) -> Result<Struct> {
    let mut fields = [("struct_type", None), ("generics", None), ("fields", None),
                      ("fields_stripped", None)];
    try!(collect_object(json, &mut fields, "Struct"));
    
    let struct_type = try!(struct_type(&fields[0].1.unwrap()));
    let generics = try!(generics(&fields[1].1.unwrap()));
    let fields_ = try!(item_datas(&fields[2].1.unwrap(), map));
    let stripped = try!(collect_bool(&fields[3].1.unwrap(), "Struct", "fields_stripped"));
    Ok(Struct { 
        struct_type: struct_type,
        generics: generics,
        fields: fields_,
        private_fields: stripped,
    })
}

fn item_enum(fields: &Slice, map: &mut ItemMap) -> Result<Item> {
    if fields.len() != 1 { error!("enum item with {} fields", fields.len()); }
    let enum_ = try!(enum_(&fields[0], map));
    Ok(Item::Enum(enum_))
}

fn enum_(json: &Value, map: &mut ItemMap) -> Result<Enum> {
    let mut fields = [("variants", None), ("generics", None)];
    try!(collect_object(json, &mut fields, "Enum"));
    let variants = try!(item_datas(fields[0].1.unwrap(), map));
    let generics = try!(generics(&fields[1].1.unwrap()));
    Ok(Enum {
        variants: variants,
        generics: generics,
    })
}

fn item_func(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("func item with {} fields", fields.len()); }
    let func = try!(func(&fields[0]));
    Ok(Item::Func(func))
}

fn func(json: &Value) -> Result<Func> {
    let mut fields = [("decl", None), ("generics", None), ("unsafety", None),
                      ("abi", None)];
    try!(collect_object(json, &mut fields, "Func"));
    let decl = try!(fn_decl(fields[0].1.unwrap()));
    let generics = try!(generics(fields[1].1.unwrap()));
    let unsafety = try!(collect_string(fields[2].1.unwrap(), "Func", "unsafety"));
    let abi = try!(abi(fields[3].1.unwrap()));
    let unsaf = unsafety == "Unsafe";
    Ok(Func {
        decl: decl,
        generics: generics,
        unsaf: unsaf,
        abi: abi,
    })
}

fn item_module(fields: &Slice, map: &mut ItemMap) -> Result<Item> {
    if fields.len() != 1 { error!("module item with {} fields", fields.len()); }
    let module = try!(module(&fields[0], map));
    Ok(Item::Module(module))
}

fn module(json: &Value, map: &mut ItemMap) -> Result<Module> {
    let mut fields = [("items", None)];
    try!(collect_object(json, &mut fields, "Module"));
    let items = try!(item_datas(&fields[0].1.unwrap(), map));
    Ok(Module { items: items })
}

fn item_typedef(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("typedef item with {} fields", fields.len()); }
    let typedef = try!(typedef(&fields[0]));
    Ok(Item::Typedef(typedef))
}

fn typedef(json: &Value) -> Result<Typedef> {
    let mut fields = [("type_", None), ("generics", None)];
    try!(collect_object(json, &mut fields, "Typedef"));
    let type_ = try!(type_(fields[0].1.unwrap()));
    let generics = try!(generics(fields[1].1.unwrap()));
    Ok(Typedef { type_: type_, generics: generics })
}

fn item_static(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("static item with {} fields", fields.len()); }
    let static_ = try!(static_(&fields[0]));
    Ok(Item::Static(static_))
}

fn static_(json: &Value) -> Result<Static> {
    let mut fields = [("type_", None), ("mutability", None), ("expr", None)];
    try!(collect_object(json, &mut fields, "Static"));
    let type_ = try!(type_(fields[0].1.unwrap()));
    let mutable = try!(mutability(fields[1].1.unwrap()));
    let expr = try!(collect_string(fields[2].1.unwrap(), "Static", "expr"));
    let expr = try!(expr.clone());
    Ok(Static { type_: type_, mutable: mutable, expr: expr })
}

fn item_constant(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("constant item with {} fields", fields.len()); }
    let constant = try!(constant(&fields[0]));
    Ok(Item::Constant(constant))
}

fn constant(json: &Value) -> Result<Constant> {
    let mut fields = [("type_", None), ("expr", None)];
    try!(collect_object(json, &mut fields, "Constant"));
    let type_ = try!(type_(fields[0].1.unwrap()));
    let expr = try!(collect_string(fields[1].1.unwrap(), "Constant", "expr"));
    let expr = try!(expr.clone());
    Ok(Constant { type_: type_, expr: expr })
}

fn item_trait(fields: &Slice, map: &mut ItemMap) -> Result<Item> {
    if fields.len() != 1 { error!("trait item with {} fields", fields.len()); }
    let trait_ = try!(trait_(&fields[0], map));
    Ok(Item::Trait(trait_))
}

fn trait_(json: &Value, map: &mut ItemMap) -> Result<Trait> {
    let mut fields = [("unsafety", None), ("items", None), ("generics", None),
                      ("bounds", None)];
    try!(collect_object(json, &mut fields, "Trait"));
    let unsaf = try!(unsafety(&fields[0].1.unwrap()));
    let items = try!(item_datas(&fields[1].1.unwrap(), map));
    let generics = try!(generics(&fields[2].1.unwrap()));
    let bounds = try!(ty_param_bounds(&fields[3].1.unwrap()));
    Ok(Trait {
        unsaf: unsaf,
        items: items,
        generics: generics,
        bounds: bounds,
    })
}

fn item_impl(fields: &Slice, map: &mut ItemMap) -> Result<Item> {
    if fields.len() != 1 { error!("impl item with {} fields", fields.len()); }
    let impl_ = try!(impl_(&fields[0], map));
    Ok(Item::Impl(impl_))
}

fn impl_(json: &Value, map: &mut ItemMap) -> Result<Impl> {
    let mut fields = [("unsafety", None), ("generics", None), ("trait_", None),
                      ("for_", None), ("items", None), ("derived", None),
                      ("polarity", None)];
    try!(collect_object(json, &mut fields, "Impl"));
    let unsaf = try!(unsafety(fields[0].1.unwrap()));
    let generics = try!(generics(fields[1].1.unwrap()));
    let trait_ = match *fields[2].1.unwrap() {
        Value::Null => None,
        _ => Some(try!(type_(fields[2].1.unwrap()))),
    };
    let for_ = try!(type_(fields[3].1.unwrap()));
    let items = try!(item_datas(fields[4].1.unwrap(), map));
    let derived = try!(collect_bool(fields[5].1.unwrap(), "Impl", "derived"));
    let negative = match *fields[6].1.unwrap() {
        Value::Null => None,
        _ => Some(try!(polarity(fields[6].1.unwrap()))),
    };
    Ok(Impl {
        unsaf:    unsaf,
        generics: generics,
        trait_:   trait_,
        for_:     for_,
        items:    items,
        derived:  derived,
        negative: negative,
    })
}

fn item_method_decl(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("method decl item with {} fields", fields.len()); }
    let method = try!(method(&fields[0]));
    Ok(Item::MethodDecl(method))
}

fn item_method(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("method item with {} fields", fields.len()); }
    let method = try!(method(&fields[0]));
    Ok(Item::Method(method))
}

fn method(json: &Value) -> Result<Method> {
    let mut fields = [("unsafety", None), ("decl", None), ("generics", None),
                      ("self_", None), ("abi", None)];
    try!(collect_object(json, &mut fields, "Method"));
    let unsaf = try!(unsafety(fields[0].1.unwrap()));
    let decl = try!(fn_decl(fields[1].1.unwrap()));
    let generics = try!(generics(fields[2].1.unwrap()));
    let self_ = try!(self_ty(fields[3].1.unwrap()));
    let abi = try!(abi(fields[4].1.unwrap()));
    Ok(Method {
        unsaf:    unsaf,
        decl:     decl,
        generics: generics,
        self_:    self_,
        abi:      abi,
    })
}

fn item_struct_field(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("struct field item with {} fields", fields.len()); }
    let struct_field = try!(struct_field(&fields[0]));
    Ok(Item::StructField(struct_field))
}

fn struct_field(json: &Value) -> Result<StructField> {
    let (variant, fields) = try!(collect_enum(json, "StructField"));
    match variant.as_ref() {
        b"HiddenStructField" => {
            if fields.len() != 0 { error!("HiddenStructField with {} fields",
                                          fields.len()); }
            Ok(StructField::Hidden)
        },
        b"TypedStructField" => {
            if fields.len() != 1 { error!("TypedStructField with {} fields",
                                          fields.len()); }
            let type_ = try!(type_(&fields[0]));
            Ok(StructField::Typed(type_))
        },
        _ => error!("Unexpected struct field variant: {:?}", variant),
    }
}

fn item_variant(fields: &Slice, map: &mut ItemMap) -> Result<Item> {
    if fields.len() != 1 { error!("variant item with {} fields", fields.len()); }
    let variant = try!(variant(&fields[0], map));
    Ok(Item::Variant(variant))
}

fn variant(json: &Value, map: &mut ItemMap) -> Result<Variant> {
    let mut fields = [("kind", None)];
    try!(collect_object(json, &mut fields, "Variant"));
    let kind = try!(variant_kind(fields[0].1.unwrap(), map));
    Ok(Variant { kind: kind })
}

fn variant_kind(json: &Value, map: &mut ItemMap) -> Result<VariantKind> {
    let (variant, fields) = try!(collect_enum(json, "VariantKind"));
    match variant.as_ref() {
        b"CLikeVariant" => {
            if fields.len() != 0 { error!("CLikeVariant with {} fields", fields.len()); }
            Ok(VariantKind::CLike)
        },
        b"TupleVariant" => {
            if fields.len() != 1 { error!("TupleVariant with {} fields", fields.len()); }
            let types = try!(types(&fields[0]));
            Ok(VariantKind::Tuple(types))
        },
        b"StructVariant" => {
            if fields.len() != 1 { error!("StructVariant with {} fields", fields.len()); }
            let variant_struct = try!(variant_struct(&fields[0], map));
            Ok(VariantKind::Struct(variant_struct))
        },
        _ => {
            error!("Unexpected VariantKind variant: {:?}", variant);
        },
    }
}

fn variant_struct(json: &Value, map: &mut ItemMap) -> Result<VariantStruct> {
    let mut fields = [("struct_type", None), ("fields", None), ("fields_stripped", None)];
    try!(collect_object(json, &mut fields, "VariantStruct"));
    let struct_type = try!(struct_type(fields[0].1.unwrap()));
    let fields_ = try!(item_datas(fields[1].1.unwrap(), map));
    let private_fields = try!(collect_bool(fields[2].1.unwrap(), "VariantStruct",
                                           "fields_stripped"));
    Ok(VariantStruct {
        struct_type: struct_type,
        fields: fields_,
        private_fields: private_fields,
    })
}

fn struct_type(json: &Value) -> Result<StructType> {
    let s = try!(collect_string(json, "?", "struct_type"));
    match s.as_ref() {
        b"Plain" => Ok(StructType::Plain),
        b"Tuple" | b"Newtype" => Ok(StructType::Tuple),
        b"Unit" => Ok(StructType::Unit),
        _ => error!("Unexpected StructType variant: {:?}", s),
    }
}

fn item_extern_func(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("extern func item with {} fields", fields.len()); }
    let func = try!(func(&fields[0]));
    Ok(Item::ExternFunc(func))
}

fn item_extern_static(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("extern static item with {} fields", fields.len()); }
    let static_ = try!(static_(&fields[0]));
    Ok(Item::ExternStatic(static_))
}

fn item_macro(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("macro item with {} fields", fields.len()); }
    let macro_ = try!(macro_(&fields[0]));
    Ok(Item::Macro(macro_))
}

fn macro_(json: &Value) -> Result<Macro> {
    let mut fields = [("source", None)];
    try!(collect_object(json, &mut fields, "Macro"));
    let source = try!(collect_string(fields[0].1.unwrap(), "Macro", "source"));
    let source = try!(source.clone());
    Ok(Macro { source: source })
}

fn item_primitive(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("primitive item with {} fields", fields.len()); }
    let primitive = try!(primitive(&fields[0]));
    Ok(Item::Primitive(primitive))
}

fn item_assoc_type(fields: &Slice) -> Result<Item> {
    if fields.len() != 2 { error!("associated type item with {} fields", fields.len()); }
    let ty_param_bounds = try!(ty_param_bounds(&fields[0]));
    let ty = match fields[1] {
        Value::Null => None,
        _ => Some(try!(type_(&fields[1]))),
    };
    Ok(Item::AssocType(ty_param_bounds, ty))
}

fn item_default_impl(fields: &Slice) -> Result<Item> {
    if fields.len() != 1 { error!("default impl item with {} fields", fields.len()); }
    let default_impl = try!(default_impl(&fields[0]));
    Ok(Item::DefaultImpl(default_impl))
}

fn default_impl(json: &Value) -> Result<DefaultImpl> {
    let mut fields = [("unsafety", None), ("trait_", None)];
    try!(collect_object(json, &mut fields, "DefaultImpl"));
    let unsaf = try!(unsafety(fields[0].1.unwrap()));
    let ty = try!(type_(fields[1].1.unwrap()));
    Ok(DefaultImpl { unsaf: unsaf, trait_: ty })
}

fn unsafety(json: &Value) -> Result<bool> {
    let string = try!(collect_string(json, "?", "unsafety"));
    Ok(string == "Unsafe")
}

fn generics(json: &Value) -> Result<Generics> {
    let mut fields = [("lifetimes", None), ("type_params", None),
                      ("where_predicates", None)];
    try!(collect_object(json, &mut fields, "Generics"));
    Ok(Generics {
        lifetimes:        try!(lifetimes(fields[0].1.unwrap())),
        type_params:      try!(ty_params(fields[1].1.unwrap())),
        where_predicates: try!(where_predicates(fields[2].1.unwrap())),
    })
}

fn ty_params(json: &Value) -> Result<SVec<TyParam>> {
    let array = try!(collect_array(json, "?", "ty_params"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(ty_param(l)));
    }
    Ok(vec)
}

fn ty_param(json: &Value) -> Result<TyParam> {
    let mut fields = [("name", None), ("did", None), ("bounds", None),
                      ("default", None)];
    try!(collect_object(json, &mut fields, "TyParam"));
    let name = try!(collect_string(fields[0].1.unwrap(), "TyParam", "name"));
    let default = match *fields[3].1.unwrap() {
        Value::Null => None,
        _ => Some(try!(type_(fields[3].1.unwrap()))),
    };
    Ok(TyParam {
        name: try!(name.clone()),
        definition: try!(def_id(fields[1].1.unwrap())),
        bounds: try!(ty_param_bounds(fields[2].1.unwrap())),
        default: default,
    })
}

fn where_predicates(json: &Value) -> Result<SVec<WherePredicate>> {
    let array = try!(collect_array(json, "?", "where_predicates"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(where_predicate(l)));
    }
    Ok(vec)
}

fn where_predicate(json: &Value) -> Result<WherePredicate> {
    let (variant, fields) = try!(collect_enum(json, "WherePredicate"));
    match variant.as_ref() {
        b"BoundPredicate" => {
            if fields.len() != 2 { error!("BoundPredicate with {} fields", fields.len()); }
            let ty = try!(type_(&fields[0]));
            let bounds = try!(ty_param_bounds(&fields[1]));
            Ok(WherePredicate::Bound(ty, bounds))
        },
        b"RegionPredicate" => {
            if fields.len() != 2 { error!("RegionPredicate with {} fields", fields.len()); }
            let lt = try!(lifetime(&fields[0]));
            let lts = try!(lifetimes(&fields[1]));
            Ok(WherePredicate::Region(lt, lts))
        },
        b"EqPredicate" => {
            if fields.len() != 2 { error!("EqPredicate with {} fields", fields.len()); }
            let lhs = try!(type_(&fields[0]));
            let rhs = try!(type_(&fields[1]));
            Ok(WherePredicate::Eq(lhs, rhs))
        },
        _ => error!("Unexpected WherePredicate variant: {:?}", variant),
    }
}

fn abi(json: &Value) -> Result<Abi> {
    let s = try!(collect_string(json, "?", "abi"));
    match s.as_ref() {
        b"Rust"          => Ok(Abi::Rust),
        b"C"             => Ok(Abi::C),
        b"System"        => Ok(Abi::System),
        b"RustIntrinsic" => Ok(Abi::RustIntrinsic),
        b"RustCall"      => Ok(Abi::RustCall),
        _ => error!("Unexpected Abi variant: {:?}", s),
    }
}

fn fn_decl(json: &Value) -> Result<FnDecl> {
    let mut fields = [("inputs", None), ("output", None), ("attrs", None)];
    try!(collect_object(json, &mut fields, "FnDecl"));
    Ok(FnDecl {
        inputs: try!(arguments(fields[0].1.unwrap())),
        output: try!(func_ret_ty(fields[1].1.unwrap())),
        attrs: try!(attributes(fields[2].1.unwrap())),
    })
}

fn func_ret_ty(json: &Value) -> Result<FuncRetTy> {
    let (variant, fields) = try!(collect_enum(json, "FuncRetTy"));
    match variant.as_ref() {
        b"Return" => {
            if fields.len() != 1 { error!("FuncRetTy::Return with {} fields",
                                          fields.len()); }
            let ty = try!(type_(&fields[0]));
            Ok(FuncRetTy::Return(ty))
        },
        b"DefaultReturn" => {
            if fields.len() != 0 { error!("DefaultReturn with {} fields", fields.len()); }
            Ok(FuncRetTy::Unit)
        },
        b"NoReturn" => {
            if fields.len() != 0 { error!("NoReturn with {} fields", fields.len()); }
            Ok(FuncRetTy::NoReturn)
        },
        _ => error!("Unexpected FuncRetTy variant: {:?}", variant),
    }
}

fn self_ty(json: &Value) -> Result<SelfTy> {
    let (variant, fields) = try!(collect_enum(json, "SelfTy"));
    match variant.as_ref() {
        b"SelfStatic" => {
            if fields.len() != 0 { error!("SelfStatic with {} fields", fields.len()); }
            Ok(SelfTy::Static)
        },
        b"SelfValue" => {
            if fields.len() != 0 { error!("SelfValue with {} fields", fields.len()); }
            Ok(SelfTy::Value)
        },
        b"SelfBorrowed" => {
            if fields.len() != 2 { error!("SelfBorrowed with {} fields", fields.len()); }
            let lt = match fields[0] {
                Value::Null => None,
                _ => Some(try!(lifetime(&fields[0]))),
            };
            let mutable = try!(mutability(&fields[1]));
            Ok(SelfTy::Borrowed(lt, mutable))
        },
        b"SelfExplicit" => {
            if fields.len() != 1 { error!("SelfExplicit with {} fields", fields.len()); }
            let ty = try!(type_(&fields[0]));
            Ok(SelfTy::Explicit(ty))
        },
        _ => error!("Unexpected SelfTy variant: {:?}", variant),
    }
}

fn mutability(json: &Value) -> Result<bool> {
    let s = try!(collect_string(json, "?", "mutability"));
    Ok(s == "Mutable")
}

fn polarity(json: &Value) -> Result<bool> {
    let s = try!(collect_string(json, "?", "polarity"));
    Ok(s == "Negative")
}

fn arguments(json: &Value) -> Result<SVec<Argument>> {
    let mut fields = [("values", None)];
    try!(collect_object(json, &mut fields, "Arguments"));
    let array = try!(collect_array(fields[0].1.unwrap(), "?", "arguments"));
    let mut vec = try!(Vec::with_capacity(array.len()));
    for l in array {
        vec.push(try!(argument(l)));
    }
    Ok(vec)
}

fn argument(json: &Value) -> Result<Argument> {
    let mut fields = [("type_", None), ("name", None), ("id", None)];
    try!(collect_object(json, &mut fields, "Argument"));
    let ty = try!(type_(fields[0].1.unwrap()));
    let name = try!(collect_string(fields[1].1.unwrap(), "Argument", "name"));
    let id = try!(collect_int(fields[2].1.unwrap(), "Argument", "id"));
    let name = try!(name.clone());
    Ok(Argument { type_: ty, name: name, id: id as u64 })
}

fn collect_string<'a>(json: &'a Value, obj: &str, field: &str) -> Result<&'a SByteString> {
    match *json {
        Value::String(ref s) => Ok(s),
        _ => error!("field {} on {} is not a string", field, obj),
    }
}

fn collect_int(json: &Value, obj: &str, field: &str) -> Result<i64> {
    match *json {
        Value::Integer(s) => Ok(s),
        _ => error!("field {} on {} is not an integer", field, obj),
    }
}

fn collect_object<'a>(obj: &'a Value, fields: &mut [(&str, Option<&'a Value>)],
                      name: &str) -> Result {
    let obj = match *obj {
        Value::Object(ref o) => o,
        _ => error!("tried to collect fields on non-object {:?}", name),
    };

    for field in obj {
        for rfield in &mut *fields {
            if &field.0 == rfield.0 {
                rfield.1 = Some(&field.1);
            }
        }
    }

    for field in fields {
        if field.1.is_none() {
            error!("did not find field {} on {}", field.0, name);
        }
    }

    Ok(())
}

fn collect_array<'a>(json: &'a Value, obj: &str, field: &str) -> Result<&'a Slice> {
    match *json {
        Value::Array(ref s) => Ok(&s[..]),
        _ => error!("field {} on {} is not an array", field, obj),
    }
}

fn collect_bool<'a>(json: &'a Value, obj: &str, field: &str) -> Result<bool> {
    match *json {
        Value::Boolean(s) => Ok(s),
        _ => error!("field {} on {} is not a boolean", field, obj),
    }
}

fn collect_enum<'a>(json: &'a Value, obj: &str) -> Result<(&'a SByteString, &'a Slice)> {
    match *json {
        Value::String(ref s) => return Ok((s, &[])),
        _ => { },
    }

    let mut fields = [("variant", None), ("fields", None)];
    try!(collect_object(json, &mut fields, obj));
    let variant = try!(collect_string(fields[0].1.unwrap(), obj, "variant"));
    let fields = try!(collect_array(fields[1].1.unwrap(), obj, "fields"));
    Ok((variant, fields))
}
