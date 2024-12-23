use std::collections::HashMap;
use thiserror::Error;

pub mod schema;

pub use schema::*;

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
        // Clear the output directory. If it didn't exist yet, ignore the error.
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
            self.generate_module(module_path, &schema_defs);
        }
    }

    /// Generates TypeScript module bindings for the given schemas. This includes
    /// creating the necessary directories and files, and writing the type
    /// definitions and imports.
    fn generate_module(&self, module_path: &'static str, schema_defs: &[SchemaDef]) {
        // Create the directory for the module
        let module_dir =
            std::path::Path::new(&self.output_path).join(module_path.replace("::", "/"));
        std::fs::create_dir_all(&module_dir).expect("Failed to create module directory");

        // Define the output file path
        let output_file_path = module_dir.join("index.ts");

        // Initialize strings to hold imports and type definitions
        let mut imports = String::new();
        let mut body = String::new();
        let mut visited_modules = std::collections::HashSet::new();

        // Go over every exported schema in the given module.
        for schema_def in schema_defs {
            // TODO: This can be abstracted as StructDef::generate or something and
            // for each variant accordingly.
            match schema_def {
                SchemaDef::Struct(struct_def) => {
                    // Identify types from foreign modules and generate import statements
                    for field in struct_def.fields {
                        // TODO: This can be abstracted, since all Schema (enum,
                        // struct) have a module_path.
                        if let SchemaDef::Struct(struct_def) = (field.schema)() {
                            let struct_path = struct_def.module_path;
                            // If the field's module is different from the currently
                            // generated module and it hasn't been visited yet,
                            // generate an import statement.
                            if struct_path != module_path && !visited_modules.contains(struct_path)
                            {
                                visited_modules.insert(struct_path);
                                let import_path =
                                    self.compute_import_path(module_path, struct_path);
                                imports.push_str(&format!(
                                    "import {{ {} }} from '{}';\n",
                                    struct_def.name, import_path
                                ));
                            }
                        }
                    }

                    // Generate type definition for the struct
                    body.push_str(&format!("export type {} = {{\n", struct_def.name));
                    for field in struct_def.fields {
                        // Map Rust type to TypeScript type
                        let ts_type = match (field.schema)() {
                            SchemaDef::Primitive(ref prim) => {
                                self.map_primitive_to_typescript(prim)
                            }
                            SchemaDef::Struct(ref struct_type) => struct_type.name.to_string(),
                            SchemaDef::Tuple(ref tuple_schemas) => {
                                let ts_types: Vec<String> = tuple_schemas
                                    .iter()
                                    .map(|schema_fn| match schema_fn() {
                                        SchemaDef::Primitive(ref prim) => {
                                            self.map_primitive_to_typescript(prim)
                                        }
                                        SchemaDef::Struct(ref struct_type) => {
                                            struct_type.name.to_string()
                                        }
                                        _ => "any".to_string(),
                                    })
                                    .collect();
                                format!("[{}]", ts_types.join(", "))
                            }
                        };
                        // Add field definition to the type
                        body.push_str(&format!("  {}: {};\n", field.name, ts_type));
                    }
                    body.push_str("};\n");
                }
                _ => {}
            }
        }

        // Combine imports and type definitions into the final output
        let final_output = format!("{}{}", imports, body);
        std::fs::write(&output_file_path, final_output).expect("Failed to write module bindings");
    }

    /// Computes the import path for a given field module relative to the current module.
    fn compute_import_path(&self, module_path: &str, field_mod: &str) -> String {
        if field_mod.starts_with(module_path) {
            // Compute relative path if the field module is a submodule
            let relative_path = field_mod.strip_prefix(module_path).unwrap();
            format!(".{}", relative_path.replace("::", "/"))
        } else {
            // Compute relative path if the field module is in a different module
            format!("../{}", field_mod.replace("::", "/"))
        }
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
