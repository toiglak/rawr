use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    EnumDef, EnumRepr, GenericDef, PrimitiveDef, Schema, SchemaDef, Shape, StructDef, VariantDef,
};

type StringCow = Cow<'static, str>;

pub struct Codegen {
    // NOTE: We're using BTreeSet instead of HashSet to ensure a consistent
    // ordering of the generated bindings (e.g., order of the generated imports),
    // which is important for the snapshot tests.
    //
    // In case the order of the generated bindings still changes (e.g., due to
    // *probably* nondeterministic `SchemaPtr` function pointer in `SchemaDef`),
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
        let mut modules: HashMap<&'static str, BTreeSet<SchemaDef>> = HashMap::new();
        for schema in &self.schemas {
            match schema {
                SchemaDef::Primitive(..)
                | SchemaDef::Sequence(..)
                | SchemaDef::Tuple(..)
                | SchemaDef::GenericParameter(..) => {}
                SchemaDef::Struct(struct_def) => {
                    modules
                        .entry(struct_def.module_path)
                        .or_insert_with(BTreeSet::new)
                        .insert(schema.generic_schema().unwrap_or(*schema));
                }
                SchemaDef::Enum(enum_def) => {
                    modules
                        .entry(enum_def.module_path)
                        .or_insert_with(BTreeSet::new)
                        .insert(schema.generic_schema().unwrap_or(*schema));
                }
            }
        }

        for (module_path, definitions) in modules {
            self.generate_module(
                module_path,
                &definitions.iter().cloned().collect::<Vec<_>>(),
            );
        }
    }

    fn generate_module(&self, module_path: &'static str, definitions: &[SchemaDef]) {
        let module_dir = self.create_module_directory(module_path);
        let output_file_path = module_dir.join("index.ts");

        let imports = self.generate_imports(module_path, definitions);
        let defs = self.generate_definitions(definitions);

        let file_content = format!("{}{}", imports, defs);
        fs::write(&output_file_path, file_content).expect("Failed to write module bindings");
    }

    fn create_module_directory(&self, module_path: &str) -> PathBuf {
        let module_dir = Path::new(&self.output_path).join(module_path.replace("::", "/"));
        fs::create_dir_all(&module_dir).expect("Failed to create module directory");
        module_dir
    }

    fn generate_imports(&self, module_path: &str, schema_defs: &[SchemaDef]) -> String {
        //// Create a list of all type dependencies that are not in this module

        type Imports = BTreeSet<SchemaDef>;

        let mut dependencies: Imports = BTreeSet::new();

        fn visit(dependencies: &mut Imports, def: SchemaDef, module_path: &&str) {
            // If the type depends on other types, for example `T` in `Option<T>`
            // or `T` and `U` in the tuple `(T, U)`, add them as a dependency
            for dep in def.generic_dependencies() {
                visit(dependencies, dep.get(), module_path);
            }

            // If the type is not in the same module, add it as a dependency
            if let Some(schema_module) = def.module_path() {
                if *module_path != schema_module {
                    // If the type is generic, add the generic schema to avoid
                    // duplicate imports (BTreeSet will filter out duplicates)
                    match def.generic_schema() {
                        Some(generic_def) => dependencies.insert(generic_def),
                        None => dependencies.insert(def),
                    };
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

    fn generate_definitions(&self, schema_defs: &[SchemaDef]) -> String {
        let mut buf = String::new();
        for schema in schema_defs {
            self.generate_definition(&mut buf, schema);
        }
        buf
    }

    /// Example of a definition.
    ///
    /// ```typescript
    /// // This whole block is a definition.
    /// export type Example = {
    ///     a: string;
    ///     b: Option<string>;
    /// };
    /// ```
    fn generate_definition(&self, buf: &mut String, schema: &SchemaDef) {
        match schema {
            SchemaDef::Struct(struct_def) => self.generate_struct_definition(struct_def, buf),
            SchemaDef::Enum(enum_def) => self.generate_enum_definition(enum_def, buf),
            _ => {}
        }
    }

    /// Example of a type.
    ///
    /// ```typescript
    /// export type Example = {
    ///      a: string; // <- `string` is a type
    ///      b: Option<string>; // <- `Option<string>` is a type
    /// };
    /// ```
    fn generate_type(&self, schema: SchemaDef) -> StringCow {
        match schema {
            SchemaDef::Primitive(ref prim) => self.primitive_to_type(prim).into(),
            SchemaDef::Sequence(ref schema) => {
                let ty = self.generate_type(schema.get());
                format!("{}[]", ty).into()
            }
            SchemaDef::Tuple(ref schemas) => {
                let ts_types: Vec<StringCow> = schemas
                    .iter()
                    .map(|schema| self.generate_type(schema.get()))
                    .collect();
                format!("[{}]", ts_types.join(", ")).into()
            }
            SchemaDef::Struct(ref struct_type) => {
                let generics = self.generate_generic_params(&struct_type.generic);
                format!("{}{}", struct_type.name, generics).into()
            }
            SchemaDef::Enum(enum_def) => {
                let generics = self.generate_generic_params(&enum_def.generic);
                format!("{}{}", enum_def.name, generics).into()
            }
            SchemaDef::GenericParameter(param) => param.into(),
        }
    }

    fn primitive_to_type(&self, primitive: &PrimitiveDef) -> &'static str {
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

    fn generate_generic_params(&self, generic: &Option<GenericDef>) -> String {
        if let Some(generic) = generic {
            let mut param_names = Vec::new();

            for param in generic.params {
                param_names.push(self.generate_type(param.get()));
            }

            if !param_names.is_empty() {
                format!("<{}>", param_names.join(", "))
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    }

    fn generate_struct_definition(&self, struct_def: &StructDef, buf: &mut String) {
        let generics = self.generate_generic_params(&struct_def.generic);
        match struct_def.shape {
            Shape::Unit => {
                buf.push_str(&format!(
                    "export type {}{} = null;\n",
                    struct_def.name, generics
                ));
            }
            Shape::Newtype(ref schema) => {
                let ty = self.generate_type(schema.get());
                buf.push_str(&format!(
                    "export type {}{} = {};\n",
                    struct_def.name, generics, ty
                ));
            }
            Shape::Tuple(ref fields) => {
                let ts_types: Vec<StringCow> = fields
                    .iter()
                    .map(|schema| self.generate_type(schema.get()))
                    .collect();
                buf.push_str(&format!(
                    "export type {}{} = [{}];\n",
                    struct_def.name,
                    generics,
                    ts_types.join(", ")
                ));
            }
            Shape::Map(ref fields) => {
                buf.push_str(&format!(
                    "export type {}{} = {{\n",
                    struct_def.name, generics
                ));
                for field in *fields {
                    let ty = self.generate_type(field.schema.get());
                    buf.push_str(&format!("  {}: {};\n", field.name, ty));
                }
                buf.push_str("};\n");
            }
        }
    }

    fn generate_enum_definition(&self, enum_def: &EnumDef, buf: &mut String) {
        let generics = self.generate_generic_params(&enum_def.generic);
        buf.push_str(&format!("export type {}{} =\n", enum_def.name, generics));
        for variant in enum_def.variants {
            buf.push_str(&self.generate_enum_variant(&enum_def.representation, variant));
        }
        buf.push_str(";\n");
    }

    fn generate_enum_variant(&self, repr: &EnumRepr, variant: &VariantDef) -> String {
        match variant.shape {
            Shape::Unit => match repr {
                EnumRepr::External => format!("  | \"{}\"\n", variant.name),
                EnumRepr::Adjacent { tag, content: _ } => {
                    format!("  | {{ {}: \"{}\" }}\n", tag, variant.name)
                }
            },
            Shape::Newtype(ref schema) => {
                let ty = self.generate_type(schema.get());
                match repr {
                    EnumRepr::External => format!("  | {{ \"{}\": {} }}\n", variant.name, ty),
                    EnumRepr::Adjacent { tag, content } => {
                        format!(
                            "  | {{ {}: \"{}\"; {}: {} }}\n",
                            tag, variant.name, content, ty
                        )
                    }
                }
            }
            Shape::Tuple(ref fields) => {
                let ts_types: Vec<StringCow> = fields
                    .iter()
                    .map(|schema| self.generate_type(schema.get()))
                    .collect();
                match repr {
                    EnumRepr::External => {
                        format!(
                            "  | {{ \"{}\": [{}] }}\n",
                            variant.name,
                            ts_types.join(", ")
                        )
                    }
                    EnumRepr::Adjacent { tag, content } => {
                        format!(
                            "  | {{ {}: \"{}\"; {}: [{}] }}\n",
                            tag,
                            variant.name,
                            content,
                            ts_types.join(", ")
                        )
                    }
                }
            }
            Shape::Map(ref fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|field| {
                        let ty = self.generate_type(field.schema.get());
                        format!("{}: {}", field.name, ty)
                    })
                    .collect();
                let fields_str = field_strs.join(", ");
                match repr {
                    EnumRepr::External => {
                        format!("  | {{ \"{}\": {{ {} }} }}\n", variant.name, fields_str)
                    }
                    EnumRepr::Adjacent { tag, content } => {
                        format!(
                            "  | {{ {}: \"{}\"; {}: {{ {} }} }}\n",
                            tag, variant.name, content, fields_str
                        )
                    }
                }
            }
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
