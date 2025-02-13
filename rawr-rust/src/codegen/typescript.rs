use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use crate::{EnumDef, EnumRepr, PrimitiveDef, Schema, SchemaDef, SchemaFn, Shape, StructDef};

type StringCow = Cow<'static, str>;

pub struct Codegen {
    // NOTE: We're using BTreeSet instead of HashSet to ensure a consistent
    // ordering of the generated bindings (e.g., order of the generated imports),
    // which is important for the snapshot tests.
    //
    // In case the order of the generated bindings still changes (e.g., due to
    // *probably* nondeterministic `SchemaFn` function pointer in `SchemaDef`),
    // consider sorting beforehand by `name`, etc.
    schemas: BTreeSet<SchemaDef>,

    output_path: PathBuf,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            output_path: PathBuf::new(),
            schemas: BTreeSet::new(),
        }
    }

    /// Export the schema from a type. All of its dependent schemas will be exported as well.
    pub fn export_type<T: Schema>(mut self) -> Self {
        self.export_schema(T::schema());
        self
    }

    /// Export schema. All of its dependent schemas will be exported as well.
    pub fn export_schema(&mut self, schema: SchemaDef) {
        // Skip if the schema has already been exported
        if self.schemas.contains(&schema) {
            return;
        }

        self.export_dependencies(schema);
        self.schemas.insert(schema);
    }

    /// Recursively export all type dependencies of a schema
    fn export_dependencies(&mut self, schema: SchemaDef) {
        schema.visit_dependencies(|dep| self.export_schema(dep));
    }

    pub fn export_to(mut self, output_path: impl AsRef<Path>) -> Self {
        self.output_path = output_path.as_ref().to_path_buf();
        self
    }

    pub fn run(self) {
        // Clear the output directory. If it didn't exist yet, ignore the error.
        let _ = fs::remove_dir_all(&self.output_path);

        // Group schemas by module
        let mut modules: HashMap<&'static str, Vec<SchemaDef>> = HashMap::new();
        for schema in &self.schemas {
            match schema {
                SchemaDef::Primitive(..) | SchemaDef::Array(..) | SchemaDef::Tuple(..) => {}
                SchemaDef::Struct(struct_def) => {
                    modules
                        .entry(struct_def.module_path)
                        .or_insert_with(Vec::new)
                        .push(schema.clone());
                }
                SchemaDef::Enum(enum_def) => {
                    modules
                        .entry(enum_def.module_path)
                        .or_insert_with(Vec::new)
                        .push(schema.clone());
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

        let imports = self.generate_imports(module_path, schema_defs);
        let body = self.generate_body(schema_defs);

        let final_output = format!("{}{}", imports, body);
        fs::write(&output_file_path, final_output).expect("Failed to write module bindings");
    }

    fn generate_imports(&self, module_path: &str, schema_defs: &[SchemaDef]) -> String {
        //// Create a list of all type dependencies that are not in this module

        let mut dependencies = BTreeSet::new();

        fn visit(dependencies: &mut BTreeSet<SchemaDef>, def: SchemaDef, module_path: &&str) {
            // If the type contains other types as part of its definition, visit them
            for dep_def in def.get_generic_dependencies() {
                visit(dependencies, dep_def(), module_path);
            }

            // If the type is not in the same module, add it as a dependency
            if let Some(schema_module) = def.module_path() {
                if *module_path != schema_module {
                    dependencies.insert(def);
                }
            }
        }

        for schema in schema_defs {
            schema.visit_dependencies(|dep| visit(&mut dependencies, dep, &module_path));
        }

        //// Generate import statements

        let mut imports = String::new();

        for dep in dependencies {
            imports.push_str(&format!(
                "import {{ type {} }} from \"{}\";\n",
                dep.name().unwrap(),
                compute_relative_path_from_module(module_path, dep.module_path().unwrap())
            ));
        }

        imports
    }

    fn generate_body(&self, schema_defs: &[SchemaDef]) -> String {
        let mut body = String::new();
        for schema in schema_defs {
            match schema {
                SchemaDef::Struct(struct_def) => {
                    self.generate_struct_body(struct_def, &mut body);
                }
                SchemaDef::Enum(enum_def) => {
                    self.generate_enum_body(enum_def, &mut body);
                }
                _ => {}
            }
        }
        body
    }

    fn create_module_directory(&self, module_path: &str) -> PathBuf {
        let module_dir = Path::new(&self.output_path).join(module_path.replace("::", "/"));
        fs::create_dir_all(&module_dir).expect("Failed to create module directory");
        module_dir
    }

    fn generate_struct_body(&self, struct_def: &StructDef, body: &mut String) {
        match struct_def.shape {
            Shape::Unit => {
                body.push_str(&format!("export type {} = null;\n", struct_def.name));
            }
            Shape::Newtype(ref schema) => {
                let ts_type = self.map_schema_to_type(schema);
                body.push_str(&format!("export type {} = {};\n", struct_def.name, ts_type));
            }
            Shape::Tuple(ref fields) => {
                let ts_types: Vec<StringCow> = fields
                    .iter()
                    .map(|schema| self.map_schema_to_type(schema))
                    .collect();
                body.push_str(&format!(
                    "export type {} = [{}];\n",
                    struct_def.name,
                    ts_types.join(", ")
                ));
            }
            Shape::Map(ref fields) => {
                body.push_str(&format!("export type {} = {{\n", struct_def.name));
                for field in *fields {
                    let ts_type = self.map_schema_to_type(&field.schema);
                    body.push_str(&format!("  {}: {};\n", field.name, ts_type));
                }
                body.push_str("};\n");
            }
        }
    }

    fn generate_enum_body(&self, enum_def: &EnumDef, body: &mut String) {
        body.push_str(&format!("export type {} =\n", enum_def.name));
        let (tag, content) = match enum_def.representation {
            EnumRepr::Adjacent { tag, content } => (tag, content),
        };
        for variant in enum_def.variants {
            match variant.shape {
                Shape::Unit => {
                    body.push_str(&format!("  | {{ {tag}: \"{}\" }}\n", variant.name));
                }
                Shape::Newtype(ref schema) => {
                    let ts_type = self.map_schema_to_type(schema);
                    body.push_str(&format!(
                        "  | {{ {tag}: \"{}\"; {content}: {} }}\n",
                        variant.name, ts_type
                    ));
                }
                Shape::Tuple(ref fields) => {
                    if fields.len() == 1 {
                        let ts_type = self.map_schema_to_type(&fields[0]);
                        body.push_str(&format!(
                            "  | {{ {tag}: \"{}\"; {content}: {} }}\n",
                            variant.name, ts_type
                        ));
                    } else {
                        let ts_types: Vec<StringCow> = fields
                            .iter()
                            .map(|schema| self.map_schema_to_type(schema))
                            .collect();
                        body.push_str(&format!(
                            "  | {{ {tag}: \"{}\"; {content}: [{}] }}\n",
                            variant.name,
                            ts_types.join(", ")
                        ));
                    }
                }
                Shape::Map(ref fields) => {
                    body.push_str(&format!(
                        "  | {{ {tag}: \"{}\"; {content}: {{\n",
                        variant.name
                    ));
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

    fn map_schema_to_type(&self, schema: &SchemaFn) -> StringCow {
        match schema() {
            SchemaDef::Primitive(ref prim) => self.map_primitive_to_type(prim).into(),
            SchemaDef::Array(ref schema) => {
                let ty = self.map_schema_to_type(schema);
                format!("{}[]", ty).into()
            }
            SchemaDef::Tuple(ref schemas) => {
                let ts_types: Vec<StringCow> = schemas
                    .iter()
                    .map(|schema| self.map_schema_to_type(schema))
                    .collect();
                format!("[{}]", ts_types.join(", ")).into()
            }
            SchemaDef::Struct(ref struct_type) => struct_type.name.into(),
            SchemaDef::Enum(enum_def) => enum_def.name.into(),
        }
    }

    fn map_primitive_to_type(&self, primitive: &PrimitiveDef) -> &'static str {
        match primitive {
            PrimitiveDef::U8
            | PrimitiveDef::U16
            | PrimitiveDef::U32
            | PrimitiveDef::U64
            | PrimitiveDef::I8
            | PrimitiveDef::I16
            | PrimitiveDef::I32
            | PrimitiveDef::I64
            | PrimitiveDef::F32
            | PrimitiveDef::F64 => "number",
            PrimitiveDef::Unit => "null",
            PrimitiveDef::Bool => "boolean",
            PrimitiveDef::Char => "string",
            PrimitiveDef::String => "string",
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
