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
    Tuple(&'static [SchemaFn]),
    Struct(StructDef),
    Enum(EnumDef),
}

impl SchemaDef {
    pub fn is_primitive(&self) -> bool {
        matches!(self, SchemaDef::Primitive(_))
    }

    pub fn is_tuple(&self) -> bool {
        matches!(self, SchemaDef::Tuple(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, SchemaDef::Struct(_))
    }

    pub fn is_enum(&self) -> bool {
        matches!(self, SchemaDef::Enum(_))
    }

    pub fn as_tuple(&self) -> Option<&'static [SchemaFn]> {
        match self {
            SchemaDef::Tuple(fields) => Some(fields),
            _ => None,
        }
    }

    pub fn name(&self) -> Option<&'static str> {
        match self {
            SchemaDef::Primitive(t) => Some(match t {
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
            SchemaDef::Tuple(_) => None,
            SchemaDef::Struct(def) => Some(def.name),
            SchemaDef::Enum(def) => Some(def.name),
        }
    }

    pub fn module_path(&self) -> Option<&'static str> {
        match self {
            SchemaDef::Primitive(_) => None,
            SchemaDef::Tuple(_) => None,
            SchemaDef::Struct(def) => Some(def.module_path),
            SchemaDef::Enum(def) => Some(def.module_path),
        }
    }

    pub fn visit_dependencies(&self, mut f: impl FnMut(SchemaDef)) {
        match self {
            SchemaDef::Primitive(_) => {}
            SchemaDef::Tuple(fields) => {
                for schema_fn in *fields {
                    let schema = schema_fn();
                    f(schema);
                }
            }
            SchemaDef::Struct(struct_def) => match struct_def.fields {
                Fields::Unit => {}
                Fields::Unnamed(fields) => {
                    for field in fields {
                        let schema = (field)();
                        f(schema);
                    }
                }
                Fields::Named(fields) => {
                    for field in fields {
                        let schema = (field.schema)();
                        f(schema);
                    }
                }
            },
            SchemaDef::Enum(enum_def) => {
                for variant in enum_def.variants {
                    match variant {
                        EnumVariant::Unit { .. } => {}
                        EnumVariant::Tuple { fields, .. } => {
                            for schema_fn in *fields {
                                let schema = schema_fn();
                                f(schema);
                            }
                        }
                        EnumVariant::Struct { fields, .. } => {
                            for field in *fields {
                                let schema = (field.schema)();
                                f(schema);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PrimitiveType {
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
  String => String,
  i32 => I32,
  i64 => I64,
  u8 => U8,
  u16 => U16,
  u32 => U32,
  u64 => U64,
  i8 => I8,
  i16 => I16,
  f32 => F32,
  f64 => F64,
  bool => Bool,
  char => Char
);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldDef {
    pub name: &'static str,
    pub schema: SchemaFn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StructDef {
    pub name: &'static str,
    pub module_path: &'static str,
    pub fields: Fields,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Fields {
    Unit,
    Unnamed(&'static [SchemaFn]),
    Named(&'static [FieldDef]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EnumDef {
    pub name: &'static str,
    pub module_path: &'static str,
    pub representation: EnumRepresentation,
    pub variants: &'static [EnumVariant],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EnumRepresentation {
    Adjacent {
        tag: &'static str,
        content: &'static str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EnumVariant {
    Unit {
        name: &'static str,
    },
    Tuple {
        name: &'static str,
        fields: &'static [SchemaFn],
    },
    Struct {
        name: &'static str,
        fields: &'static [FieldDef],
    },
}
