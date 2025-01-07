use rawr::codegen::typescript;

pub mod enumeration;
pub mod module;
pub mod structure;

pub fn export_to(path: &str) {
    typescript::Codegen::new()
        .export_type::<structure::Structure>()
        .export_to(path)
        .run()
}
