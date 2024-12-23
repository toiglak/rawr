pub use linkme::*;
use std::{any::TypeId, collections::HashMap};
use thiserror::Error;

/////////// Services /////////////

pub type Result<T> = std::result::Result<T, TransportError>;

#[derive(Debug, Clone, Error)]
pub enum TransportError {
    #[error("Failed to send data")]
    SendError,
    #[error("Failed to receive data")]
    ReceiveError,
    #[error("Connection closed")]
    Closed,
}

/////////// Schemas /////////////

#[distributed_slice]
pub static SCHEMA_REGISTRY: [fn() -> SchemaDef];

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

#[derive(Debug, Clone)]
pub enum SchemaDef {
    Primitive(PrimitiveType),
    Struct(StructDef),
}

pub trait Schema {
    fn schema() -> SchemaDef;
}

impl Schema for String {
    fn schema() -> SchemaDef {
        SchemaDef::Primitive(PrimitiveType::String)
    }
}

impl Schema for i32 {
    fn schema() -> SchemaDef {
        SchemaDef::Primitive(PrimitiveType::I32)
    }
}

impl Schema for bool {
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
    pub name: &'static str,
    pub module_path: &'static str,
    pub fields: Vec<FieldDef>,
}

#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: &'static str,
    pub schema: fn() -> SchemaDef,
}

/////////// Code Generation /////////////

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
        println!("Registry length: {:?}", SCHEMA_REGISTRY.len());
        self
    }

    pub fn run(self) {
        // Clear the output directory
        // Ignore error if the path didn't exist.
        let _ = std::fs::remove_dir_all(&self.output_path);

        // Group schemas by module
        let mut modules: HashMap<&'static str, Vec<SchemaDef>> = HashMap::new();
        for schema_fn in SCHEMA_REGISTRY {
            let schema_def = schema_fn();
            if let SchemaDef::Struct(struct_def) = &schema_def {
                modules
                    .entry(struct_def.module_path)
                    .or_insert_with(Vec::new)
                    .push(schema_def.clone());
            }
        }

        for (module_path, schema_defs) in modules {
            self.generate_module_bindings(module_path, &schema_defs);
        }
    }

    fn generate_module_bindings(&self, module_path: &'static str, schema_defs: &[SchemaDef]) {
        let module_dir =
            std::path::Path::new(&self.output_path).join(module_path.replace("::", "/"));
        std::fs::create_dir_all(&module_dir).expect("Failed to create module directory");
        let output_file_path = module_dir.join("index.ts");

        let mut imports = String::new();
        let mut body = String::new();
        let mut visited_modules = std::collections::HashSet::new();

        for schema_def in schema_defs {
            if let SchemaDef::Struct(struct_def) = schema_def {
                // Collect cross-module references
                for field in &struct_def.fields {
                    if let SchemaDef::Struct(field_struct_def) = (field.schema)() {
                        let field_mod = field_struct_def.module_path;
                        if field_mod != module_path && !visited_modules.contains(field_mod) {
                            visited_modules.insert(field_mod);
                            let import_path = if field_mod.starts_with(module_path) {
                                let relative_path = field_mod.strip_prefix(module_path).unwrap();
                                format!(".{}", relative_path.replace("::", "/"))
                            } else {
                                format!("../{}", field_mod.replace("::", "/"))
                            };
                            imports.push_str(&format!(
                                "import {{ {} }} from '{}';\n",
                                field_struct_def.name, import_path
                            ));
                        }
                    }
                }
                // Generate type definition
                body.push_str(&format!("export type {} = {{\n", struct_def.name));
                for field in &struct_def.fields {
                    let ts_type = match (field.schema)() {
                        SchemaDef::Primitive(ref prim) => self.map_primitive_to_typescript(prim),
                        SchemaDef::Struct(ref struct_type) => struct_type.name.to_string(),
                    };
                    body.push_str(&format!("  {}: {};\n", field.name, ts_type));
                }
                body.push_str("};\n");
            }
        }

        let final_output = format!("{}{}", imports, body);
        std::fs::write(&output_file_path, final_output).expect("Failed to write module bindings");
    }

    fn map_primitive_to_typescript(&self, primitive: &PrimitiveType) -> String {
        match primitive {
            PrimitiveType::String => "string".to_string(),
            PrimitiveType::I32
            | PrimitiveType::I64
            | PrimitiveType::U32
            | PrimitiveType::U64
            | PrimitiveType::F32
            | PrimitiveType::F64 => "number".to_string(),
            PrimitiveType::Bool => "boolean".to_string(),
            _ => "any".to_string(),
        }
    }
}
