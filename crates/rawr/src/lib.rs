use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
};
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
        let _ = fs::remove_dir_all(&self.output_path);

        // Group schemas by module
        let mut modules: HashMap<&'static str, Vec<SchemaDef>> = HashMap::new();
        for schema in SCHEMA_REGISTRY {
            match schema() {
                SchemaDef::Primitive(..) | SchemaDef::Tuple(..) => {}
                SchemaDef::Struct(struct_def) => {
                    modules
                        .entry(struct_def.module_path)
                        .or_insert_with(Vec::new)
                        .push(schema().clone());
                }
                SchemaDef::Enum(enum_def) => {
                    modules
                        .entry(enum_def.module_path)
                        .or_insert_with(Vec::new)
                        .push(schema().clone());
                }
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
        fs::write(&output_file_path, final_output).expect("Failed to write module bindings");
    }

    fn create_module_directory(&self, module_path: &str) -> PathBuf {
        let module_dir = Path::new(&self.output_path).join(module_path.replace("::", "/"));
        fs::create_dir_all(&module_dir).expect("Failed to create module directory");
        module_dir
    }

    fn generate_imports_and_body(
        &self,
        module_path: &str,
        schema_defs: &[SchemaDef],
    ) -> (String, String) {
        let mut imports = String::new();
        let mut body = String::new();
        let mut visited_modules = HashSet::new();

        for schema_def in schema_defs {
            match schema_def {
                SchemaDef::Primitive(..) | SchemaDef::Tuple(..) => {}
                SchemaDef::Struct(struct_def) => {
                    self.generate_imports_for_struct(
                        module_path,
                        struct_def,
                        &mut imports,
                        &mut visited_modules,
                    );
                    self.generate_struct_body(struct_def, &mut body);
                }
                SchemaDef::Enum(enum_def) => {
                    self.generate_imports_for_enum(
                        module_path,
                        enum_def,
                        &mut imports,
                        &mut visited_modules,
                    );
                    self.generate_enum_body(enum_def, &mut body);
                }
            }
        }

        (imports, body)
    }

    fn generate_imports_for_struct(
        &self,
        current_module: &str,
        struct_def: &StructDef,
        imports: &mut String,
        visited_modules: &mut HashSet<&str>,
    ) {
        // Identify types from foreign modules and generate import statements
        for field in struct_def.fields {
            // TODO: This can be abstracted, since almost all
            // Schema-s (enum, struct) have a module_path.
            if let SchemaDef::Struct(struct_def) = (field.schema)() {
                // If the field's module is different from the currently generated
                // module and it hasn't been visited yet, generate an import
                // statement.
                let struct_module = struct_def.module_path;
                if struct_module != current_module && !visited_modules.contains(struct_module) {
                    visited_modules.insert(struct_module);
                    let import_path =
                        compute_relative_path_from_module(current_module, struct_module);
                    imports.push_str(&format!(
                        "import {{ {} }} from '{}';\n",
                        struct_def.name, import_path
                    ));
                }
            }
        }
    }

    fn generate_imports_for_enum(
        &self,
        current_module: &str,
        enum_def: &EnumDef,
        imports: &mut String,
        visited_modules: &mut HashSet<&str>,
    ) {
        for variant in enum_def.variants {
            match variant {
                EnumVariant::Tuple { fields, .. } => {
                    for field in *fields {
                        if let SchemaDef::Struct(struct_def) = (field)() {
                            let struct_module = struct_def.module_path;
                            if struct_module != current_module
                                && !visited_modules.contains(struct_module)
                            {
                                visited_modules.insert(struct_module);
                                let import_path = compute_relative_path_from_module(
                                    current_module,
                                    struct_module,
                                );
                                imports.push_str(&format!(
                                    "import {{ {} }} from '{}';\n",
                                    struct_def.name, import_path
                                ));
                            }
                        }
                    }
                }
                EnumVariant::Struct { fields, .. } => {
                    for field in *fields {
                        if let SchemaDef::Struct(struct_def) = (field.schema)() {
                            let struct_module = struct_def.module_path;
                            if struct_module != current_module
                                && !visited_modules.contains(struct_module)
                            {
                                visited_modules.insert(struct_module);
                                let import_path = compute_relative_path_from_module(
                                    current_module,
                                    struct_module,
                                );
                                imports.push_str(&format!(
                                    "import {{ {} }} from '{}';\n",
                                    struct_def.name, import_path
                                ));
                            }
                        }
                    }
                }
                _ => {}
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

    fn generate_enum_body(&self, enum_def: &EnumDef, body: &mut String) {
        body.push_str(&format!("export type {} =\n", enum_def.name));
        for variant in enum_def.variants {
            match variant {
                EnumVariant::Unit { name } => {
                    body.push_str(&format!("  | {{ type: \"{}\" }}\n", name));
                }
                EnumVariant::Tuple { name, fields } => {
                    if fields.len() == 1 {
                        let ts_type = self.map_schema_to_type(&fields[0]);
                        body.push_str(&format!(
                            "  | {{ type: \"{}\"; data: {} }}\n",
                            name, ts_type
                        ));
                    } else {
                        let ts_types: Vec<String> = fields
                            .iter()
                            .map(|schema| self.map_schema_to_type(schema))
                            .collect();
                        body.push_str(&format!(
                            "  | {{ type: \"{}\"; data: [{}] }}\n",
                            name,
                            ts_types.join(", ")
                        ));
                    }
                }
                EnumVariant::Struct { name, fields } => {
                    body.push_str(&format!("  | {{ type: \"{}\"; data: {{\n", name));
                    for field in *fields {
                        let ts_type = self.map_schema_to_type(&field.schema);
                        body.push_str(&format!("    {}: {};\n", field.name, ts_type));
                    }
                    body.push_str("  } }\n");
                }
            }
        }
        body.push_str(";\n");
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
            SchemaDef::Enum(enum_def) => enum_def.name.to_string(),
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
}

/// Computes relative typescript import path from `current` to `target` rust module.
fn compute_relative_path_from_module(current: &str, target: &str) -> String {
    let current_segments: Vec<&str> = current.split("::").collect();
    let target_segments: Vec<&str> = target.split("::").collect();
    let mut i = 0;
    while i < current_segments.len()
        && i < target_segments.len()
        && current_segments[i] == target_segments[i]
    {
        i += 1;
    }
    let leftover_current = &current_segments[i..];
    let leftover_target = &target_segments[i..];
    let up = "../".repeat(leftover_current.len());
    let down = leftover_target.join("/");

    if down.is_empty() {
        if up.is_empty() {
            ".".to_string()
        } else {
            up.trim_end_matches('/').to_string()
        }
    } else {
        format!("{}{}", if up.is_empty() { "./" } else { &up }, down)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_compute_relative_file_path_from_module() {
        let f = compute_relative_path_from_module;

        assert_eq!(f("crate_name",                 "crate_name::module::nested"), "./module/nested");
        assert_eq!(f("crate_name::module::nested", "crate_name::module"),         "..");
        assert_eq!(f("crate_name::module::nested", "crate_name"),                 "../..");
        assert_eq!(f("crate_name",                 "other_crate"),                "../other_crate");
        assert_eq!(f("crate_name::module",         "other_crate::module"),        "../../other_crate/module");
    }
}
