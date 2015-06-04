// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::rc::{Arc};
use lrs::share::{RefCell};
use lrs::vec::{Vec};
use lrs::string::{ByteString};
use lrs::bx::{Box};

use markup::{Document};

pub struct Crate {
    pub item: Arc<ItemData>,
}

pub struct ItemData {
    pub name: Option<ByteString>,
    pub attrs: Vec<Attribute>,
    pub docs: Document,
    pub inner: Item,
    pub public: bool,
    pub node: DefId,
    pub parent: RefCell<Option<Arc<ItemData>>>,
    pub impls: RefCell<Vec<Arc<ItemData>>>,
}

pub enum Attribute {
    Word(ByteString),
    List(ByteString, Vec<Attribute>),
    NameValue(ByteString, ByteString),
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
    AssocType(AssocType),
    DefaultImpl(DefaultImpl),
}

pub struct AssocType {
    pub bounds: Vec<TyParamBound>,
    pub default: Option<Type>,
}

pub struct GlobImport {
    pub path: Path,
    pub node: Option<DefId>,
}

pub struct Struct {
    pub struct_type: StructType,
    pub generics: Generics,
    pub fields: Vec<Arc<ItemData>>,
    pub private_fields: bool,
}

#[derive(Eq)]
pub enum StructType {
    Plain,
    Tuple,
    Unit,
}

pub struct Generics {
    pub lifetimes: Vec<ByteString>,
    pub type_params: Vec<TyParam>,
    pub where_predicates: Vec<WherePredicate>
}

pub struct TypeBinding {
    pub name: ByteString,
    pub ty: Type,
}

pub enum PathParameters {
    AngleBracketed(AngleBracketedPathParams),
    Parenthesized(ParenthesizedPathParams),
}

pub struct AngleBracketedPathParams {
    pub lifetimes: Vec<ByteString>,
    pub ty_params: Vec<Type>,
    pub bindings: Vec<TypeBinding>,
}

pub struct ParenthesizedPathParams {
    pub args: Vec<Type>,
    pub return_value: Option<Type>,
}

pub struct PathSegment {
    pub name: ByteString,
    pub params: PathParameters
}

pub struct Path {
    pub global: bool,
    pub segments: Vec<PathSegment>,
}

#[derive(Copy, Eq)]
pub struct DefId {
    pub node: u64,
    pub krate: u64,
}

pub enum Type {
    ResolvedPath(ResolvedPath),
    Generic(Generic),
    Primitive(Primitive),
    BareFunction(BareFunction),
    Tuple(Tuple),
    Slice(Slice),
    Array(Array),
    Bottom,
    Pointer(Pointer),
    Ref(Ref),
    UfcsPath(UfcsPath),
    Infer,
    HkltBound(HkltBound),
}

pub struct ResolvedPath {
    pub path: Path,
    pub params: Option<Vec<TyParamBound>>,
    pub def_id: DefId,
    pub is_generic: bool,
    pub item: RefCell<Option<Arc<ItemData>>>,
}

pub struct Generic {
    pub name: ByteString,
}

pub struct BareFunction {
    pub decl: Box<BareFunctionDecl>,
}

pub struct Slice {
    pub ty: Box<Type>,
}

pub struct Tuple {
    pub fields: Vec<Type>,
}

pub struct Array {
    pub ty: Box<Type>,
    pub initializer: ByteString,
}

pub struct Pointer {
    pub mutable: bool,
    pub ty: Box<Type>,
}

pub struct Ref {
    pub lifetime: Option<ByteString>,
    pub mutable: bool,
    pub ty: Box<Type>,
}

pub struct UfcsPath {
    pub target: ByteString,
    pub self_ty: Box<Type>,
    pub trait_: Box<Type>,
}

pub struct HkltBound {
    pub bounds: Vec<TyParamBound>,
}

pub struct TyParam {
    pub name: ByteString,
    pub definition: DefId,
    pub bounds: Vec<TyParamBound>,
    pub default: Option<Type>,
}

pub enum TyParamBound {
    Lifetime(ByteString),
    Trait(TraitTyParamBound),
}

pub struct TraitTyParamBound {
    pub trait_: PolyTrait,
    pub maybe: bool,
}

pub struct PolyTrait {
    pub trait_: Type,
    pub lifetimes: Vec<ByteString>
}

pub enum WherePredicate {
    Bound(BoundWherePredicate),
    Region(RegionWherePredicate),
    Eq(EqWherePredicate),
}

pub struct BoundWherePredicate {
    pub ty: Type,
    pub bounds: Vec<TyParamBound>,
}

pub struct RegionWherePredicate {
    pub lt: ByteString,
    pub bounds: Vec<ByteString>,
}

pub struct EqWherePredicate {
    pub lhs: Type,
    pub rhs: Type,
}

pub struct Macro {
    pub source: ByteString,
}

pub struct Static {
    pub type_: Type,
    pub mutable: bool,
    pub expr: ByteString,
}

pub struct DefaultImpl {
    pub unsaf: bool,
    pub trait_: Type,
}

#[derive(Copy)]
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
    pub abi: ByteString,
}

pub struct FnDecl {
    pub inputs: Vec<Argument>,
    pub output: FuncRetTy,
    pub attrs: Vec<Attribute>,
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
    Tuple(Vec<Type>),
    Struct(VariantStruct),
}

pub struct VariantStruct {
    pub struct_type: StructType,
    pub fields: Vec<Arc<ItemData>>,
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
    pub items: Vec<Arc<ItemData>>,
    pub derived: bool,
    pub negative: Option<bool>,
}

pub enum SelfTy {
    Static,
    Value,
    Borrowed(Option<ByteString>, bool),
    Explicit(Type),
}

pub struct Argument {
    pub type_: Type,
    pub name: ByteString,
    pub id: u64,
}

pub struct Constant {
    pub type_: Type,
    pub expr: ByteString,
}

pub enum FuncRetTy {
    Return(Type),
    Unit,
    NoReturn
}

pub struct Trait {
    pub unsaf: bool,
    pub items: Vec<Arc<ItemData>>,
    pub generics: Generics,
    pub bounds: Vec<TyParamBound>,
}

pub struct Typedef {
    pub type_: Type,
    pub generics: Generics,
    pub is_assoc: bool,
}

pub struct Enum {
    pub variants: Vec<Arc<ItemData>>,
    pub generics: Generics,
}

pub struct Module {
    pub items: Vec<Arc<ItemData>>,
}


pub trait Walker: Sized {
    fn walk_crate                       (&mut self, val: &Crate                    ) { walk_crate                       (self, val) }
    fn walk_item_data                   (&mut self, val: &Arc<ItemData>            ) { walk_item_data                   (self, val) }
    fn walk_attribute                   (&mut self, val: &Attribute                ) { walk_attribute                   (self, val) }
    fn walk_item                        (&mut self, val: &Item                     ) { walk_item                        (self, val) }
    fn walk_glob_import                 (&mut self, val: &GlobImport               ) { walk_glob_import                 (self, val) }
    fn walk_struct                      (&mut self, val: &Struct                   ) { walk_struct                      (self, val) }
    fn walk_struct_type                 (&mut self, val: &StructType               ) { walk_struct_type                 (self, val) }
    fn walk_generics                    (&mut self, val: &Generics                 ) { walk_generics                    (self, val) }
    fn walk_assoc_type                  (&mut self, val: &AssocType                ) { walk_assoc_type                  (self, val) }
    fn walk_resolved_path               (&mut self, val: &ResolvedPath             ) { walk_resolved_path               (self, val) }
    fn walk_generic                     (&mut self, val: &Generic                  ) { walk_generic                     (self, val) }
    fn walk_bare_function               (&mut self, val: &BareFunction             ) { walk_bare_function               (self, val) }
    fn walk_type_binding                (&mut self, val: &TypeBinding              ) { walk_type_binding                (self, val) }
    fn walk_tuple                       (&mut self, val: &Tuple                    ) { walk_tuple                       (self, val) }
    fn walk_slice                       (&mut self, val: &Slice                    ) { walk_slice                       (self, val) }
    fn walk_array                       (&mut self, val: &Array                    ) { walk_array                       (self, val) }
    fn walk_pointer                     (&mut self, val: &Pointer                  ) { walk_pointer                     (self, val) }
    fn walk_ref                         (&mut self, val: &Ref                      ) { walk_ref                         (self, val) }
    fn walk_ufcs_path                   (&mut self, val: &UfcsPath                 ) { walk_ufcs_path                   (self, val) }
    fn walk_hklt_bound                  (&mut self, val: &HkltBound                ) { walk_hklt_bound                  (self, val) }
    fn walk_trait_ty_param_bound        (&mut self, val: &TraitTyParamBound        ) { walk_trait_ty_param_bound        (self, val) }
    fn walk_bound_where_predicate       (&mut self, val: &BoundWherePredicate      ) { walk_bound_where_predicate       (self, val) }
    fn walk_region_where_predicate      (&mut self, val: &RegionWherePredicate     ) { walk_region_where_predicate      (self, val) }
    fn walk_eq_where_predicate          (&mut self, val: &EqWherePredicate         ) { walk_eq_where_predicate          (self, val) }
    fn walk_path_parameters             (&mut self, val: &PathParameters           ) { walk_path_parameters             (self, val) }
    fn walk_angle_bracketed_path_params (&mut self, val: &AngleBracketedPathParams ) { walk_angle_bracketed_path_params (self, val) }
    fn walk_parenthesized_path_params   (&mut self, val: &ParenthesizedPathParams  ) { walk_parenthesized_path_params   (self, val) }
    fn walk_path_segment                (&mut self, val: &PathSegment              ) { walk_path_segment                (self, val) }
    fn walk_path                        (&mut self, val: &Path                     ) { walk_path                        (self, val) }
    fn walk_def_id                      (&mut self, val: &DefId                    ) { walk_def_id                      (self, val) }
    fn walk_type                        (&mut self, val: &Type                     ) { walk_type                        (self, val) }
    fn walk_ty_param                    (&mut self, val: &TyParam                  ) { walk_ty_param                    (self, val) }
    fn walk_ty_param_bound              (&mut self, val: &TyParamBound             ) { walk_ty_param_bound              (self, val) }
    fn walk_poly_trait                  (&mut self, val: &PolyTrait                ) { walk_poly_trait                  (self, val) }
    fn walk_where_predicate             (&mut self, val: &WherePredicate           ) { walk_where_predicate             (self, val) }
    fn walk_macro                       (&mut self, val: &Macro                    ) { walk_macro                       (self, val) }
    fn walk_static                      (&mut self, val: &Static                   ) { walk_static                      (self, val) }
    fn walk_default_impl                (&mut self, val: &DefaultImpl              ) { walk_default_impl                (self, val) }
    fn walk_primitive                   (&mut self, val: &Primitive                ) { walk_primitive                   (self, val) }
    fn walk_bare_function_decl          (&mut self, val: &BareFunctionDecl         ) { walk_bare_function_decl          (self, val) }
    fn walk_fn_decl                     (&mut self, val: &FnDecl                   ) { walk_fn_decl                     (self, val) }
    fn walk_func                        (&mut self, val: &Func                     ) { walk_func                        (self, val) }
    fn walk_abi                         (&mut self, val: &Abi                      ) { walk_abi                         (self, val) }
    fn walk_variant                     (&mut self, val: &Variant                  ) { walk_variant                     (self, val) }
    fn walk_variant_kind                (&mut self, val: &VariantKind              ) { walk_variant_kind                (self, val) }
    fn walk_variant_struct              (&mut self, val: &VariantStruct            ) { walk_variant_struct              (self, val) }
    fn walk_struct_field                (&mut self, val: &StructField              ) { walk_struct_field                (self, val) }
    fn walk_method                      (&mut self, val: &Method                   ) { walk_method                      (self, val) }
    fn walk_impl                        (&mut self, val: &Impl                     ) { walk_impl                        (self, val) }
    fn walk_self_ty                     (&mut self, val: &SelfTy                   ) { walk_self_ty                     (self, val) }
    fn walk_argument                    (&mut self, val: &Argument                 ) { walk_argument                    (self, val) }
    fn walk_constant                    (&mut self, val: &Constant                 ) { walk_constant                    (self, val) }
    fn walk_func_ret_ty                 (&mut self, val: &FuncRetTy                ) { walk_func_ret_ty                 (self, val) }
    fn walk_trait                       (&mut self, val: &Trait                    ) { walk_trait                       (self, val) }
    fn walk_typedef                     (&mut self, val: &Typedef                  ) { walk_typedef                     (self, val) }
    fn walk_enum                        (&mut self, val: &Enum                     ) { walk_enum                        (self, val) }
    fn walk_module                      (&mut self, val: &Module                   ) { walk_module                      (self, val) }
}

/// pub struct Crate {
///     pub item: Arc<ItemData>,
/// }
pub fn walk_crate             <W: Walker> ( w: &mut W , val: &Crate             ) {
    w.walk_item_data(&val.item);
}

/// pub struct ItemData {
///     pub name: Option<ByteString>,
///     pub attrs: Vec<Attribute>,
///     pub docs: Document,
///     pub inner: Item,
///     pub public: bool,
///     pub node: DefId,
/// }
pub fn walk_item_data         <W: Walker> ( w: &mut W , val: &Arc<ItemData>     ) {
    w.walk_item(&val.inner)
}

/// pub enum Attribute {
///     Word(ByteString),
///     List(ByteString, Vec<Attribute>),
///     NameValue(ByteString, ByteString),
/// }
pub fn walk_attribute         <W: Walker> ( _: &mut W , _: &Attribute         ) {
}

/// pub enum Item {
///     GlobImport(GlobImport),
///     Struct(Struct),
///     Enum(Enum),
///     Func(Func),
///     Module(Module),
///     Typedef(Typedef),
///     Static(Static),
///     Constant(Constant),
///     Trait(Trait),
///     Impl(Impl),
///     MethodDecl(Method),
///     Method(Method),
///     StructField(StructField),
///     Variant(Variant),
///     ExternFunc(Func),
///     ExternStatic(Static),
///     Macro(Macro),
///     Primitive(Primitive),
///     AssocType(AssocType),
///     DefaultImpl(DefaultImpl),
/// }
pub fn walk_item              <W: Walker> ( w: &mut W , val: &Item              ) {
    match *val {
        Item::GlobImport   (ref g) => w.walk_glob_import  (g),
        Item::Struct       (ref s) => w.walk_struct       (s),
        Item::Enum         (ref e) => w.walk_enum         (e),
        Item::Func         (ref f) => w.walk_func         (f),
        Item::Module       (ref m) => w.walk_module       (m),
        Item::Typedef      (ref t) => w.walk_typedef      (t),
        Item::Static       (ref s) => w.walk_static       (s),
        Item::Constant     (ref c) => w.walk_constant     (c),
        Item::Trait        (ref t) => w.walk_trait        (t),
        Item::Impl         (ref i) => w.walk_impl         (i),
        Item::MethodDecl   (ref m) => w.walk_method       (m),
        Item::Method       (ref m) => w.walk_method       (m),
        Item::StructField  (ref s) => w.walk_struct_field (s),
        Item::Variant      (ref v) => w.walk_variant      (v),
        Item::ExternFunc   (ref e) => w.walk_func         (e),
        Item::ExternStatic (ref e) => w.walk_static       (e),
        Item::Macro        (ref m) => w.walk_macro        (m),
        Item::Primitive    (ref p) => w.walk_primitive    (p),
        Item::AssocType    (ref a) => w.walk_assoc_type   (a),
        Item::DefaultImpl  (ref i) => w.walk_default_impl (i),
    }
}

/// pub struct GlobImport {
///     pub path: Path,
///     pub node: Option<DefId>,
/// }
pub fn walk_glob_import       <W: Walker> ( w: &mut W , val: &GlobImport        ) {
    w.walk_path(&val.path)
}

/// pub struct Struct {
///     pub struct_type: StructType,
///     pub generics: Generics,
///     pub fields: Vec<Arc<ItemData>>,
///     pub private_fields: bool,
/// }
pub fn walk_struct            <W: Walker> ( w: &mut W , val: &Struct            ) {
    w.walk_struct_type(&val.struct_type);
    w.walk_generics(&val.generics);
    for field in &val.fields {
        w.walk_item_data(field);
    }
}

/// pub enum StructType {
///     Plain,
///     Tuple,
///     Unit,
/// }
pub fn walk_struct_type       <W: Walker> ( _: &mut W , _: &StructType        ) {
}

/// pub struct Generics {
///     pub lifetimes: Vec<ByteString>,
///     pub type_params: Vec<TyParam>,
///     pub where_predicates: Vec<WherePredicate>
/// }
pub fn walk_generics          <W: Walker> ( w: &mut W , val: &Generics          ) {
    for param in &val.type_params {
        w.walk_ty_param(param);
    }
    for pred in &val.where_predicates {
        w.walk_where_predicate(pred);
    }
}

///pub struct AssocType {
///    pub bounds: Vec<TyParamBound>,
///    pub default: Option<Type>,
///}
pub fn walk_assoc_type<W: Walker> ( w: &mut W , val: &AssocType ) {
    for bound in &val.bounds {
        w.walk_ty_param_bound(bound);
    }
    if let Some(ref d) = val.default {
        w.walk_type(d);
    }
}

/// pub struct ResolvedPath {
///     pub path: Path,
///     pub params: Option<Vec<TyParamBound>>,
///     pub def_id: DefId,
///     pub item: RefCell<Option<Arc<ItemData>>>,
/// }
pub fn walk_resolved_path<W: Walker> ( w: &mut W , val: &ResolvedPath ) {
    w.walk_path(&val.path);
    if let Some(ref params) = val.params {
        for param in params {
            w.walk_ty_param_bound(param);
        }
    }
    w.walk_def_id(&val.def_id);
}

/// pub struct Generic {
///     pub name: ByteString,
/// }
pub fn walk_generic<W: Walker> ( _: &mut W , _: &Generic ) {
}

/// pub struct BareFunction {
///     pub decl: Box<BareFunctionDecl>,
/// }
pub fn walk_bare_function<W: Walker> ( w: &mut W , val: &BareFunction ) {
    w.walk_bare_function_decl(&val.decl);
}

/// pub struct TypeBinding {
///     pub name: ByteString,
///     pub ty: Type,
/// }
pub fn walk_type_binding      <W: Walker> ( w: &mut W , val: &TypeBinding       ) {
    w.walk_type(&val.ty);
}

/// pub struct Tuple {
///     pub fields: Vec<Type>,
/// }
pub fn walk_tuple<W: Walker> ( w: &mut W , val: &Tuple ) {
    for field in &val.fields {
        w.walk_type(field);
    }
}

/// pub struct Slice {
///     pub ty: Box<Type>,
/// }
pub fn walk_slice<W: Walker> ( w: &mut W , val: &Slice ) {
    w.walk_type(&val.ty);
}

/// pub struct Array {
///     pub ty: Box<Type>,
///     pub initializer: ByteString,
/// }
pub fn walk_array<W: Walker> ( w: &mut W, val: &Array ) {
    w.walk_type(&val.ty);
}

/// pub struct Pointer {
///     pub mutable: bool,
///     pub ty: Box<Type>,
/// }
pub fn walk_pointer<W: Walker> ( w: &mut W , val: &Pointer ) {
    w.walk_type(&val.ty);
}

/// pub struct Ref {
///     pub lifetime: Option<ByteString>,
///     pub mutable: bool,
///     pub ty: Box<Type>,
/// }
pub fn walk_ref<W: Walker> ( w: &mut W , val: &Ref ) {
    w.walk_type(&val.ty);
}

/// pub struct UfcsPath {
///     pub target: ByteString,
///     pub self_ty: Box<Type>,
///     pub trait_: Box<Type>,
/// }
pub fn walk_ufcs_path<W: Walker> ( w: &mut W , val: &UfcsPath ) {
    w.walk_type(&val.self_ty);
    w.walk_type(&val.trait_);
}

/// pub struct HkltBound {
///     pub bounds: Vec<TyParamBound>,
/// }
pub fn walk_hklt_bound<W: Walker> ( w: &mut W , val: &HkltBound) {
    for bound in &val.bounds {
        w.walk_ty_param_bound(bound);
    }
}

/// pub struct TraitTyParamBound {
///     pub trait_: PolyTrait,
///     pub maybe: bool,
/// }
pub fn walk_trait_ty_param_bound<W: Walker> ( w: &mut W , val: &TraitTyParamBound ) {
    w.walk_poly_trait(&val.trait_);
}

///pub struct BoundWherePredicate {
///    pub ty: Type,
///    pub bounds: Vec<TyParamBound>,
///}
pub fn walk_bound_where_predicate<W: Walker> ( w: &mut W , val: &BoundWherePredicate ) {
    w.walk_type(&val.ty);
    for bound in &val.bounds {
        w.walk_ty_param_bound(bound);
    }
}

/// pub struct RegionWherePredicate {
///     pub lt: ByteString,
///     pub bounds: Vec<ByteString>,
/// }
pub fn walk_region_where_predicate<W: Walker> ( _: &mut W , _: &RegionWherePredicate ) {
}

/// pub struct EqWherePredicate {
///     pub lhs: Type,
///     pub rhs: Type,
/// }
pub fn walk_eq_where_predicate<W: Walker> ( w: &mut W , val: &EqWherePredicate ) {
    w.walk_type(&val.lhs);
    w.walk_type(&val.rhs);
}

/// pub enum PathParameters {
///     AngleBracketed(AngleBracketedPathParam),
///     Parenthesized(ParenthesizedPathParam),
/// }
pub fn walk_path_parameters   <W: Walker> ( w: &mut W , val: &PathParameters    ) {
    match *val {
        PathParameters::AngleBracketed(ref a) => w.walk_angle_bracketed_path_params(a),
        PathParameters::Parenthesized(ref p) => w.walk_parenthesized_path_params(p),
    }
}

/// pub struct AngleBracketedPathParam {
///     pub lifetimes: Vec<ByteString>,
///     pub ty_params: Vec<Type>,
///     pub bindings: Vec<TypeBinding>,
/// }
pub fn walk_angle_bracketed_path_params<W: Walker> ( w: &mut W , val: &AngleBracketedPathParams ) {
    for p in &val.ty_params {
        w.walk_type(p);
    }
    for b in &val.bindings {
        w.walk_type_binding(b);
    }
}

/// pub struct ParenthesizedPathParam {
///     pub args: Vec<Type>,
///     pub return_value: Option<Type>,
/// }
pub fn walk_parenthesized_path_params<W: Walker> ( w: &mut W, val: &ParenthesizedPathParams ) {
    for a in &val.args {
        w.walk_type(a);
    }
    if let Some(ref t) = val.return_value {
        w.walk_type(t);
    }
}

/// pub struct PathSegment {
///     pub name: ByteString,
///     pub params: PathParameters
/// }
pub fn walk_path_segment      <W: Walker> ( w: &mut W , val: &PathSegment       ) {
    w.walk_path_parameters(&val.params);
}

/// pub struct Path {
///     pub global: bool,
///     pub segments: Vec<PathSegment>,
/// }
pub fn walk_path              <W: Walker> ( w: &mut W , val: &Path              ) {
    for segment in &val.segments {
        w.walk_path_segment(segment);
    }
}

/// pub struct DefId {
///     pub node: u64,
///     pub krate: u64,
/// }
pub fn walk_def_id            <W: Walker> ( _: &mut W , _: &DefId             ) {
}

/// pub enum Type {
///     ResolvedPath(ResolvedPath),
///     Generic(Generic),
///     Primitive(Primitive),
///     BareFunction(BareFunction),
///     Tuple(Tuple),
///     Slice(Slice),
///     Array(Array),
///     Bottom,
///     Pointer(Pointer),
///     Ref(Ref),
///     UfcsPath(UfcsPath),
///     Infer,
///     HkltBound(HkltBound),
/// }
pub fn walk_type              <W: Walker> ( w: &mut W , val: &Type              ) {
    match *val {
        Type::ResolvedPath(ref r) => w.walk_resolved_path(r),
        Type::Generic(ref g) => w.walk_generic(g),
        Type::Primitive(ref p) => w.walk_primitive(p),
        Type::BareFunction(ref b) => w.walk_bare_function(b),
        Type::Tuple(ref t) => w.walk_tuple(t),
        Type::Slice(ref s) => w.walk_slice(s),
        Type::Array(ref a) => w.walk_array(a),
        Type::Bottom => { },
        Type::Pointer(ref p) => w.walk_pointer(p),
        Type::Ref(ref r) => w.walk_ref(r),
        Type::UfcsPath(ref u) => w.walk_ufcs_path(u),
        Type::Infer => { },
        Type::HkltBound(ref h) => w.walk_hklt_bound(h),
    }
}

/// pub struct TyParam {
///     pub name: ByteString,
///     pub definition: DefId,
///     pub bounds: Vec<TyParamBound>,
///     pub default: Option<Type>,
/// }
pub fn walk_ty_param          <W: Walker> ( w: &mut W , val: &TyParam           ) {
    w.walk_def_id(&val.definition);
    for bound in &val.bounds {
        w.walk_ty_param_bound(bound);
    }
    if let Some(ref d) = val.default {
        w.walk_type(d);
    }
}

/// pub enum TyParamBound {
///     Lifetime(ByteString),
///     Trait(TraitTyParamBound),
/// }
pub fn walk_ty_param_bound    <W: Walker> ( w: &mut W , val: &TyParamBound      ) {
    if let TyParamBound::Trait(ref t) = *val {
        w.walk_trait_ty_param_bound(t);
    }
}

/// pub struct PolyTrait {
///     pub trait_: Type,
///     pub lifetimes: Vec<ByteString>
/// }
pub fn walk_poly_trait        <W: Walker> ( w: &mut W , val: &PolyTrait         ) {
    w.walk_type(&val.trait_);
}

/// pub enum WherePredicate {
///     Bound(BoundWherePredicate),
///     Region(RegionWherePredicate),
///     Eq(EqWherePredicate),
/// }
pub fn walk_where_predicate   <W: Walker> ( w: &mut W , val: &WherePredicate    ) {
    match *val {
        WherePredicate::Bound(ref b) => w.walk_bound_where_predicate(b),
        WherePredicate::Region(ref r) => w.walk_region_where_predicate(r),
        WherePredicate::Eq(ref e) => w.walk_eq_where_predicate(e),
    }
}

/// pub struct Macro {
///     pub source: ByteString,
/// }
pub fn walk_macro             <W: Walker> ( _: &mut W , _: &Macro             ) {
}

/// pub struct Static {
///     pub type_: Type,
///     pub mutable: bool,
///     pub expr: ByteString,
/// }
pub fn walk_static            <W: Walker> ( w: &mut W , val: &Static            ) {
    w.walk_type(&val.type_);
}

/// pub struct DefaultImpl {
///     pub unsaf: bool,
///     pub trait_: Type,
/// }
pub fn walk_default_impl      <W: Walker> ( w: &mut W , val: &DefaultImpl       ) {
    w.walk_type(&val.trait_);
}

/// pub enum Primitive {
///     Isize, I8, I16, I32, I64,
///     Usize, U8, U16, U32, U64,
///     F32, F64,
///     Char,
///     Bool,
///     Str,
///     Slice,
///     Array,
///     Tuple,
///     RawPointer,
/// }
pub fn walk_primitive         <W: Walker> ( _: &mut W , _: &Primitive         ) {
}

/// pub struct BareFunctionDecl {
///     pub unsaf: bool,
///     pub generics: Generics,
///     pub decl: FnDecl,
///     pub abi: ByteString,
/// }
pub fn walk_bare_function_decl<W: Walker> ( w: &mut W , val: &BareFunctionDecl  ) {
    w.walk_generics(&val.generics);
    w.walk_fn_decl(&val.decl);
}

/// pub struct FnDecl {
///     pub inputs: Vec<Argument>,
///     pub output: FuncRetTy,
///     pub attrs: Vec<Attribute>,
/// }
pub fn walk_fn_decl           <W: Walker> ( w: &mut W , val: &FnDecl            ) {
    for arg in &val.inputs {
        w.walk_argument(arg);
    }
    w.walk_func_ret_ty(&val.output);
    for attr in &val.attrs {
        w.walk_attribute(attr);
    }
}

/// pub struct Func {
///     pub decl: FnDecl,
///     pub generics: Generics,
///     pub unsaf: bool,
///     pub abi: Abi
/// }
pub fn walk_func              <W: Walker> ( w: &mut W , val: &Func              ) {
    w.walk_fn_decl(&val.decl);
    w.walk_generics(&val.generics);
    w.walk_abi(&val.abi);
}

/// pub enum Abi {
///     Rust,
///     C,
///     System,
///     RustIntrinsic,
///     RustCall,
/// }
pub fn walk_abi               <W: Walker> ( _: &mut W , _: &Abi               ) {
}

/// pub struct Variant {
///     pub kind: VariantKind,
/// }
pub fn walk_variant           <W: Walker> ( w: &mut W , val: &Variant           ) {
    w.walk_variant_kind(&val.kind);
}

/// pub enum VariantKind {
///     CLike,
///     Tuple(Vec<Type>),
///     Struct(VariantStruct),
/// }
pub fn walk_variant_kind      <W: Walker> ( w: &mut W , val: &VariantKind       ) {
    match *val {
        VariantKind::CLike => { },
        VariantKind::Tuple(ref ts) => {
            for t in ts {
                w.walk_type(t);
            }
        },
        VariantKind::Struct(ref s) => w.walk_variant_struct(s),
    }
}

/// pub struct VariantStruct {
///     pub struct_type: StructType,
///     pub fields: Vec<Arc<ItemData>>,
///     pub private_fields: bool,
/// }
pub fn walk_variant_struct    <W: Walker> ( w: &mut W , val: &VariantStruct     ) {
    w.walk_struct_type(&val.struct_type);
    for field in &val.fields {
        w.walk_item_data(field);
    }
}

/// pub enum StructField {
///     Hidden,
///     Typed(Type),
/// }
pub fn walk_struct_field      <W: Walker> ( w: &mut W , val: &StructField       ) {
    match *val {
        StructField::Hidden => { },
        StructField::Typed(ref t) => w.walk_type(t),
    }
}

/// pub struct Method {
///     pub unsaf:    bool,
///     pub decl:     FnDecl,
///     pub generics: Generics,
///     pub self_:    SelfTy,
///     pub abi:      Abi
/// }
pub fn walk_method            <W: Walker> ( w: &mut W , val: &Method            ) {
    w.walk_fn_decl(&val.decl);
    w.walk_generics(&val.generics);
    w.walk_self_ty(&val.self_);
    w.walk_abi(&val.abi);
}

/// pub struct Impl {
///     pub unsaf: bool,
///     pub generics: Generics,
///     pub trait_: Option<Type>,
///     pub for_: Type,
///     pub items: Vec<Arc<ItemData>>,
///     pub derived: bool,
///     pub negative: Option<bool>,
/// }
pub fn walk_impl              <W: Walker> ( w: &mut W , val: &Impl         ) {
    w.walk_generics(&val.generics);
    if let Some(ref t) = val.trait_ {
        w.walk_type(t);
    }
    w.walk_type(&val.for_);
    for item in &val.items {
        w.walk_item_data(item);
    }
}

/// pub enum SelfTy {
///     Static,
///     Value,
///     Borrowed(Option<ByteString>, bool),
///     Explicit(Type),
/// }
pub fn walk_self_ty           <W: Walker> ( w: &mut W , val: &SelfTy            ) {
    match *val {
        SelfTy::Static => { },
        SelfTy::Value => { },
        SelfTy::Borrowed(_, _) => { },
        SelfTy::Explicit(ref t) => w.walk_type(t),
    }
}

/// pub struct Argument {
///     pub type_: Type,
///     pub name: ByteString,
///     pub id: u64,
/// }
pub fn walk_argument          <W: Walker> ( w: &mut W , val: &Argument          ) {
    w.walk_type(&val.type_);
}

/// pub struct Constant {
///     pub type_: Type,
///     pub expr: ByteString,
/// }
pub fn walk_constant          <W: Walker> ( w: &mut W , val: &Constant          ) {
    w.walk_type(&val.type_);
}

/// pub enum FuncRetTy {
///     Return(Type),
///     Unit,
///     NoReturn
/// }
pub fn walk_func_ret_ty       <W: Walker> ( w: &mut W , val: &FuncRetTy         ) {
    match *val {
        FuncRetTy::Return(ref t) => w.walk_type(t),
        FuncRetTy::Unit => { },
        FuncRetTy::NoReturn => { },
    }
}

/// pub struct Trait {
///     pub unsaf: bool,
///     pub items: Vec<Arc<ItemData>>,
///     pub generics: Generics,
///     pub bounds: Vec<TyParamBound>,
/// }
pub fn walk_trait             <W: Walker> ( w: &mut W , val: &Trait             ) {
    for item in &val.items {
        w.walk_item_data(item);
    }
    w.walk_generics(&val.generics);
    for bound in &val.bounds {
        w.walk_ty_param_bound(bound);
    }
}

/// pub struct Typedef {
///     pub type_: Type,
///     pub generics: Generics,
/// }
pub fn walk_typedef           <W: Walker> ( w: &mut W , val: &Typedef           ) {
    w.walk_type(&val.type_);
    w.walk_generics(&val.generics);
}

/// pub struct Enum {
///     pub variants: Vec<Arc<ItemData>>,
///     pub generics: Generics,
/// }
pub fn walk_enum              <W: Walker> ( w: &mut W , val: &Enum              ) {
    for item in &val.variants {
        w.walk_item_data(item);
    }
    w.walk_generics(&val.generics);
}

/// pub struct Module {
///     pub items: Vec<Arc<ItemData>>,
/// }
pub fn walk_module            <W: Walker> ( w: &mut W , val: &Module            ) {
    for item in &val.items {
        w.walk_item_data(item);
    }
}
