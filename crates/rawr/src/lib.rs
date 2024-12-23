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

type SchemaFn = fn() -> SchemaDef;

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
        for schema in SCHEMA_REGISTRY {
            if let SchemaDef::Struct(struct_def) = &schema() {
                modules
                    .entry(struct_def.module_path)
                    .or_insert_with(Vec::new)
                    .push(schema().clone());
            }
        }

        for (module_path, schema_defs) in modules {
            self.generate_module(module_path, &schema_defs);
        }
    }

    fn generate_module(&self, module_path: &'static str, schema_defs: &[SchemaDef]) {
        let module_dir = self.create_module_directory(module_path);
        let output_file_path = module_dir.join("index.ts");

        let (imports, body) = self.generate_imports_and_body(module_path, schema_defs);

        let final_output = format!("{}{}", imports, body);
        std::fs::write(&output_file_path, final_output).expect("Failed to write module bindings");
    }

    fn create_module_directory(&self, module_path: &str) -> std::path::PathBuf {
        let module_dir =
            std::path::Path::new(&self.output_path).join(module_path.replace("::", "/"));
        std::fs::create_dir_all(&module_dir).expect("Failed to create module directory");
        module_dir
    }

    fn generate_imports_and_body(
        &self,
        module_path: &str,
        schema_defs: &[SchemaDef],
    ) -> (String, String) {
        let mut imports = String::new();
        let mut body = String::new();
        let mut visited_modules = std::collections::HashSet::new();

        for schema_def in schema_defs {
            match schema_def {
                SchemaDef::Struct(struct_def) => {
                    self.generate_imports(
                        module_path,
                        struct_def,
                        &mut imports,
                        &mut visited_modules,
                    );
                    self.generate_struct_body(struct_def, &mut body);
                }
                _ => {}
            }
        }

        (imports, body)
    }

    fn generate_imports(
        &self,
        module_path: &str,
        struct_def: &StructDef,
        imports: &mut String,
        visited_modules: &mut std::collections::HashSet<&str>,
    ) {
        // Identify types from foreign modules and generate import statements
        for field in struct_def.fields {
            // TODO: This can be abstracted, since almost all
            // Schema-s (enum, struct) have a module_path.
            if let SchemaDef::Struct(struct_def) = (field.schema)() {
                // If the field's module is different from the currently generated
                // module and it hasn't been visited yet, generate an import
                // statement.
                let struct_path = struct_def.module_path;
                if struct_path != module_path && !visited_modules.contains(struct_path) {
                    visited_modules.insert(struct_path);
                    let import_path = self.compute_import_path(module_path, struct_path);
                    imports.push_str(&format!(
                        "import {{ {} }} from '{}';\n",
                        struct_def.name, import_path
                    ));
                }
            }
        }
    }

    fn generate_struct_body(&self, struct_def: &StructDef, body: &mut String) {
        body.push_str(&format!("export type {} = {{\n", struct_def.name));
        for field in struct_def.fields {
            let ts_type = self.map_schema_to_type(&field.schema);
            body.push_str(&format!("  {}: {};\n", field.name, ts_type));
        }
        body.push_str("};\n");
    }

    fn map_schema_to_type(&self, schema: &SchemaFn) -> String {
        match schema() {
            SchemaDef::Primitive(ref prim) => self.map_primitive_to_type(prim),
            SchemaDef::Struct(ref struct_type) => struct_type.name.to_string(),
            SchemaDef::Tuple(ref tuple_schemas) => {
                let ts_types: Vec<String> = tuple_schemas
                    .iter()
                    .map(|schema| self.map_schema_to_type(schema))
                    .collect();
                format!("[{}]", ts_types.join(", "))
            }
        }
    }

    fn map_primitive_to_type(&self, primitive: &PrimitiveType) -> String {
        match primitive {
            PrimitiveType::U8
            | PrimitiveType::U16
            | PrimitiveType::U32
            | PrimitiveType::U64
            | PrimitiveType::I8
            | PrimitiveType::I16
            | PrimitiveType::I32
            | PrimitiveType::I64
            | PrimitiveType::F32
            | PrimitiveType::F64 => "number".to_string(),
            PrimitiveType::Bool => "boolean".to_string(),
            PrimitiveType::Char => "string".to_string(),
            PrimitiveType::String => "string".to_string(),
        }
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
}
