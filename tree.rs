// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::rc::{Arc};
use lrs::cell::{RefCell};
use lrs::vec::{SVec};
use lrs::string::{SByteString};
use lrs::bx::{Box};

use markup::{Document};

pub struct Crate {
    pub item: Arc<ItemData>,
}

pub struct ItemData {
    pub name: Option<SByteString>,
    pub attrs: SVec<Attribute>,
    pub docs: Document,
    pub inner: Item,
    pub public: bool,
    pub node: u64,
}

pub enum Attribute {
    Word(SByteString),
    List(SByteString, SVec<Attribute>),
    NameValue(SByteString, SByteString),
}

pub enum Item {
    GlobImport(GlobImport),
    Struct(Struct),
    Enum(Enum),
    Func(Func),
    Module(Module),
    Typedef(Typedef),
    Static(Static),
    Constant(Constant),
    Trait(Trait),
    Impl(Impl),
    MethodDecl(Method),
    Method(Method),
    StructField(StructField),
    Variant(Variant),
    ExternFunc(Func),
    ExternStatic(Static),
    Macro(Macro),
    Primitive(Primitive),
    AssocType(SVec<TyParamBound>, Option<Type>),
    DefaultImpl(DefaultImpl),
}

pub struct GlobImport {
    pub path: Path,
    pub node: Option<u64>,
}

pub struct Struct {
    pub struct_type: StructType,
    pub generics: Generics,
    pub fields: SVec<Arc<ItemData>>,
    pub private_fields: bool,
}

pub enum StructType {
    Plain,
    Tuple,
    Unit,
}

pub struct Generics {
    pub lifetimes: SVec<SByteString>,
    pub type_params: SVec<TyParam>,
    pub where_predicates: SVec<WherePredicate>
}

pub struct TypeBinding {
    pub name: SByteString,
    pub ty: Type,
}

pub enum PathParameters {
    AngleBracketed(SVec<SByteString>, SVec<Type>, SVec<TypeBinding>),
    Parenthesized(SVec<Type>, Option<Type>),
}

pub struct PathSegment {
    pub name: SByteString,
    pub params: PathParameters
}

pub struct Path {
    pub global: bool,
    pub segments: SVec<PathSegment>,
}

pub enum Type {
    ResolvedPath(Path, Option<SVec<TyParamBound>>, u64, RefCell<Option<Arc<ItemData>>>),
    Generic(SByteString),
    Primitive(Primitive),
    BareFunction(Box<BareFunctionDecl>),
    Tuple(SVec<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, SByteString),
    Bottom,
    Pointer(bool, Box<Type>),
    Ref(Option<SByteString>, bool, Box<Type>),
    UfcsPath(SByteString, Box<Type>, Box<Type>),
    Infer,
    HkltBound(SVec<TyParamBound>),
}

pub struct TyParam {
    pub name: SByteString,
    pub definition: u64,
    pub bounds: SVec<TyParamBound>,
    pub default: Option<Type>,
}

pub enum TyParamBound {
    Lifetime(SByteString),
    Trait(PolyTrait, bool),
}

pub struct PolyTrait {
    pub trait_: Type,
    pub lifetimes: SVec<SByteString>
}

pub enum WherePredicate {
    Bound(Type, SVec<TyParamBound>),
    Region(SByteString, SVec<SByteString>),
    Eq(Type, Type),
}

pub struct Macro {
    pub source: SByteString,
}

pub struct Static {
    pub type_: Type,
    pub mutable: bool,
    pub expr: SByteString,
}

pub struct DefaultImpl {
    pub unsaf: bool,
    pub trait_: Type,
}

pub enum Primitive {
    Isize, I8, I16, I32, I64,
    Usize, U8, U16, U32, U64,
    F32, F64,
    Char,
    Bool,
    Str,
    Slice,
    Array,
    Tuple,
    RawPointer,
}

pub struct BareFunctionDecl {
    pub unsaf: bool,
    pub generics: Generics,
    pub decl: FnDecl,
    pub abi: SByteString,
}

pub struct FnDecl {
    pub inputs: SVec<Argument>,
    pub output: FuncRetTy,
    pub attrs: SVec<Attribute>,
}

pub struct Func {
    pub decl: FnDecl,
    pub generics: Generics,
    pub unsaf: bool,
    pub abi: Abi
}
pub enum Abi {
    Rust,
    C,
    System,
    RustIntrinsic,
    RustCall,
}

pub struct Variant {
    pub kind: VariantKind,
}

pub enum VariantKind {
    CLike,
    Tuple(SVec<Type>),
    Struct(VariantStruct),
}

pub struct VariantStruct {
    pub struct_type: StructType,
    pub fields: SVec<Arc<ItemData>>,
    pub private_fields: bool,
}

pub enum StructField {
    Hidden,
    Typed(Type),
}

pub struct Method {
    pub unsaf:    bool,
    pub decl:     FnDecl,
    pub generics: Generics,
    pub self_:    SelfTy,
    pub abi:      Abi
}

pub struct Impl {
    pub unsaf: bool,
    pub generics: Generics,
    pub trait_: Option<Type>,
    pub for_: Type,
    pub items: SVec<Arc<ItemData>>,
    pub derived: bool,
    pub negative: Option<bool>,
}

pub enum SelfTy {
    Static,
    Value,
    Borrowed(Option<SByteString>, bool),
    Explicit(Type),
}

pub struct Argument {
    pub type_: Type,
    pub name: SByteString,
    pub id: u64,
}

pub struct Constant {
    pub type_: Type,
    pub expr: SByteString,
}

pub enum FuncRetTy {
    Return(Type),
    Unit,
    NoReturn
}

pub struct Trait {
    pub unsaf: bool,
    pub items: SVec<Arc<ItemData>>,
    pub generics: Generics,
    pub bounds: SVec<TyParamBound>,
}

pub struct Typedef {
    pub type_: Type,
    pub generics: Generics,
}

pub struct Enum {
    pub variants: SVec<Arc<ItemData>>,
    pub generics: Generics,
}

pub struct Module {
    pub items: SVec<Arc<ItemData>>,
}
