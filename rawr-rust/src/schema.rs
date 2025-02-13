// NOTE: We use this to prevent "infinitely sized structures" in [SchemaDef],
// since it can contain itself.
//
/// Treat this as "a pointer to a schema definition".
pub type SchemaFn = fn() -> SchemaDef;

pub trait Schema {
    fn schema() -> SchemaDef;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SchemaDef {
    Primitive(PrimitiveType),
    Array(SchemaFn),
    Tuple(&'static [SchemaFn]),
    Enum(EnumDef),
    Struct(StructDef),
}

impl SchemaDef {
    /// Returns type dependencies for the generic schemas.
    ///
    /// When a type includes generics, concrete instantiations of these generics
    /// (e.g., `MyType` in `Option<MyType>`) must be imported at the point of use
    /// in the generated binding file.
    pub fn get_generic_dependencies(&self) -> &[SchemaFn] {
        match self {
            SchemaDef::Array(schema) => std::slice::from_ref(schema),
            SchemaDef::Tuple(fields) => fields,
            // TODO: Struct and Enum generics
            _ => &[],
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        match self {
            SchemaDef::Primitive(t) => Some(match t {
                PrimitiveType::Unit => "()",
                PrimitiveType::U8 => "u8",
                PrimitiveType::U16 => "u16",
                PrimitiveType::U32 => "u32",
                PrimitiveType::U64 => "u64",
                PrimitiveType::I8 => "i8",
                PrimitiveType::I16 => "i16",
                PrimitiveType::I32 => "i32",
                PrimitiveType::I64 => "i64",
                PrimitiveType::F32 => "f32",
                PrimitiveType::F64 => "f64",
                PrimitiveType::Bool => "bool",
                PrimitiveType::Char => "char",
                PrimitiveType::String => "String",
            }),
            SchemaDef::Array(_) => None,
            SchemaDef::Tuple(_) => None,
            SchemaDef::Struct(def) => Some(def.name),
            SchemaDef::Enum(def) => Some(def.name),
        }
    }

    pub fn module_path(&self) -> Option<&'static str> {
        match self {
            SchemaDef::Primitive(_) => None,
            SchemaDef::Array(_) => None,
            SchemaDef::Tuple(_) => None,
            SchemaDef::Struct(def) => Some(def.module_path),
            SchemaDef::Enum(def) => Some(def.module_path),
        }
    }

    pub fn visit_dependencies(&self, mut visit: impl FnMut(SchemaDef)) {
        match self {
            SchemaDef::Primitive(_) => {}
            SchemaDef::Array(schema) => {
                visit(schema());
            }
            SchemaDef::Tuple(fields) => {
                for schema_fn in *fields {
                    let schema = schema_fn();
                    visit(schema);
                }
            }
            SchemaDef::Struct(struct_def) => {
                struct_def.shape.visit_dependencies(&mut visit);
            }
            SchemaDef::Enum(enum_def) => {
                for variant in enum_def.variants {
                    variant.shape.visit_dependencies(&mut visit);
                }
            }
        }
    }
}

//// Primitives

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrimitiveType {
    Unit,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Char,
    String,
}

macro_rules! impl_schema_for_primitive {
    ($($t:ty => $variant:ident),*) => {
        $(
            impl Schema for $t {
                fn schema() -> SchemaDef {
                    SchemaDef::Primitive(PrimitiveType::$variant)
                }
            }
        )*
    };
}

impl_schema_for_primitive!(
  () => Unit,
  i8 => I8,
  i16 => I16,
  i32 => I32,
  i64 => I64,
  u8 => U8,
  u16 => U16,
  u32 => U32,
  u64 => U64,
  f32 => F32,
  f64 => F64,
  bool => Bool,
  char => Char,
  String => String
);

//// Array-like

impl<T: Schema> Schema for Vec<T> {
    fn schema() -> SchemaDef {
        SchemaDef::Array(T::schema)
    }
}

impl<T: Schema> Schema for [T] {
    fn schema() -> SchemaDef {
        SchemaDef::Array(T::schema)
    }
}

impl<T: Schema> Schema for &[T] {
    fn schema() -> SchemaDef {
        SchemaDef::Array(T::schema)
    }
}

impl<T: Schema> Schema for &mut [T] {
    fn schema() -> SchemaDef {
        SchemaDef::Array(T::schema)
    }
}

impl<const N: usize, T: Schema> Schema for [T; N] {
    fn schema() -> SchemaDef {
        SchemaDef::Array(T::schema)
    }
}

//// Tuples

macro_rules! impl_schema_for_tuples {
    ($(($($name:ident),+)),+) => {
        $(
            impl<$($name: Schema),+> Schema for ($($name,)+) {
                fn schema() -> SchemaDef {
                    SchemaDef::Tuple(&[$($name::schema),+])
                }
            }
        )+
    };
}

impl_schema_for_tuples!(
    (A),
    (A, B),
    (A, B, C),
    (A, B, C, D),
    (A, B, C, D, E),
    (A, B, C, D, E, F),
    (A, B, C, D, E, F, G),
    (A, B, C, D, E, F, G, H),
    (A, B, C, D, E, F, G, H, I),
    (A, B, C, D, E, F, G, H, I, J),
    (A, B, C, D, E, F, G, H, I, J, K),
    (A, B, C, D, E, F, G, H, I, J, K, L),
    (A, B, C, D, E, F, G, H, I, J, K, L, M),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O),
    (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
);

////

/// Represents the shape of a struct or enum variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Shape {
    /// Represents a unit-like struct or enum variant.
    ///
    /// Example:
    ///
    /// ```
    /// struct UnitStruct;
    /// enum E { UnitVariant }
    /// ```
    Unit,
    /// Represents a newtype-like struct or enum variant.
    ///
    /// Example:
    ///
    /// ```
    /// struct NewtypeStruct(i32);
    /// struct NewtypeStruct((i32, String));
    /// enum E { NewtypeVariant(i32) }
    /// enum E { NewtypeVariant((i32, String)) }
    /// ```
    Newtype(SchemaFn),
    /// Represents a tuple-like struct or enum variant.
    ///
    /// Example:
    ///
    /// ```
    /// struct TupleStruct();
    /// struct TupleStruct(i32, String);
    /// enum E { TupleVariant() }
    /// enum E { TupleVariant(i32, String) }
    /// ```
    Tuple(&'static [SchemaFn]),
    /// Represents a struct or enum variant with named fields.
    ///
    /// Example:
    ///
    /// ```
    /// struct MapStruct { }
    /// struct MapStruct { a: i32, b: String }
    /// enum E { MapVariant { } }
    /// enum E { MapVariant { a: i32, b: String } }
    /// ```
    Map(&'static [FieldDef]),
}

impl Shape {
    fn visit_dependencies(&self, visit: &mut impl FnMut(SchemaDef)) {
        match *self {
            Shape::Unit => {}
            Shape::Newtype(schema) => visit(schema()),
            Shape::Tuple(fields) => {
                for field in fields {
                    let schema = (field)();
                    visit(schema);
                }
            }
            Shape::Map(fields) => {
                for field in fields {
                    let schema = (field.schema)();
                    visit(schema);
                }
            }
        }
    }
}

//// Structs

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StructDef {
    pub name: &'static str,
    pub module_path: &'static str,
    pub shape: Shape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldDef {
    pub name: &'static str,
    pub schema: SchemaFn,
}

//// Enums

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnumDef {
    pub name: &'static str,
    pub module_path: &'static str,
    pub representation: EnumRepr,
    pub variants: &'static [VariantDef],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VariantDef {
    pub name: &'static str,
    pub shape: Shape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EnumRepr {
    Adjacent {
        tag: &'static str,
        content: &'static str,
    },
}
