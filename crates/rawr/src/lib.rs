pub use linkme::*;
use std::any::TypeId;

pub type Result<T> = std::result::Result<T, TransportError>;

#[derive(Debug, Clone)]
pub enum TransportError {
    SendError,
    ReceiveError,
    Closed,
}

#[distributed_slice]
pub static REGISTRY: [fn() -> SchemaDef];

#[derive(Debug, Clone)]
pub struct TypeSchema {
    pub name: String,
    pub module_path: String,
    pub crate_name: String,
    pub definition: &'static SchemaDef,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub type_id: TypeId,
    pub name: String,
    pub module_path: String,
    pub crate_name: String,
}

/// Defines the structure of a type.
#[derive(Debug, Clone)]
pub enum SchemaDef {
    Primitive(PrimitiveType),
    Struct(StructDef),
}

/// Trait for types that have a schema definition.
pub trait HasSchema {
    fn schema() -> SchemaDef;
}

// Implement HasSchema for primitive types
impl HasSchema for String {
    fn schema() -> SchemaDef {
        SchemaDef::Primitive(PrimitiveType::String)
    }
}

impl HasSchema for i32 {
    fn schema() -> SchemaDef {
        SchemaDef::Primitive(PrimitiveType::I32)
    }
}

impl HasSchema for bool {
    fn schema() -> SchemaDef {
        SchemaDef::Primitive(PrimitiveType::Bool)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone)]
pub struct StructDef {
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub schema: SchemaDef,
}

// --- Code Generation ---

pub struct Codegen {
    output_path: String,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output_path: String::new(),
        }
    }

    pub fn export_to(mut self, output_path: &str) -> Self {
        self.output_path = output_path.to_string();
        self
    }

    pub fn debug(self) -> Self {
        println!("Output path: {:?}", self.output_path);
        println!("Registry length: {:?}", REGISTRY.len());
        self
    }

    pub fn run(self) {
        let mut generated_types = std::collections::HashSet::new();

        for schema_fn in REGISTRY {
            let schema_def = schema_fn();
            self.generate_bindings(&schema_def, &mut generated_types);
        }
    }

    fn generate_bindings(
        &self,
        schema_def: &SchemaDef,
        generated_types: &mut std::collections::HashSet<String>,
    ) {
        match schema_def {
            SchemaDef::Struct(struct_def) => {
                // Assuming the struct name can be derived from the SchemaDef for now
                // This will be more robust with the actual macro output
                let struct_name = "MyData".to_string();

                if generated_types.contains(&struct_name) {
                    return;
                }
                generated_types.insert(struct_name.clone());

                let output_file_path =
                    format!("{}/{}.ts", self.output_path, struct_name.to_lowercase());
                let mut output = String::new();
                output.push_str(&format!("export type {} = {{\n", struct_name));
                for field in &struct_def.fields {
                    let ts_type = self.map_schema_def_to_typescript(&field.schema);
                    output.push_str(&format!("  {}: {};\n", field.name, ts_type));
                }
                output.push_str("};\n");
                std::fs::write(&output_file_path, output).expect("Failed to write output file");
                println!(
                    "Generated bindings for {} at {:?}",
                    struct_name, output_file_path
                );
            }
            SchemaDef::Primitive(_) => {
                // Primitives don't generate standalone files
            }
        }
    }

    fn map_schema_def_to_typescript(&self, schema_def: &SchemaDef) -> String {
        match schema_def {
            SchemaDef::Primitive(primitive) => match primitive {
                PrimitiveType::String => "string".to_string(),
                PrimitiveType::I32
                | PrimitiveType::I64
                | PrimitiveType::U32
                | PrimitiveType::U64
                | PrimitiveType::F32
                | PrimitiveType::F64 => "number".to_string(),
                PrimitiveType::Bool => "boolean".to_string(),
                _ => "any".to_string(), // Add more primitive type mappings as needed
            },
            SchemaDef::Struct(_) => "any".to_string(), // Will be replaced by the actual type name
        }
    }
}
