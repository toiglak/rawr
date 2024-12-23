#[linkme::distributed_slice]
pub static SCHEMA_REGISTRY: [fn() -> SchemaDef];

pub trait Schema {
    fn schema() -> SchemaDef;
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

#[derive(Debug, Clone, Copy)]
pub enum SchemaDef {
    Primitive(PrimitiveType),
    Struct(StructDef),
    Tuple(&'static [fn() -> SchemaDef]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy)]
pub struct StructDef {
    pub name: &'static str,
    pub module_path: &'static str,
    pub fields: &'static [FieldDef],
}

#[derive(Debug, Clone, Copy)]
pub struct FieldDef {
    pub name: &'static str,
    pub schema: fn() -> SchemaDef,
}

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
