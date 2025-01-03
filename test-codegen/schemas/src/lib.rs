pub mod enumeration;
pub mod module;
pub mod structure;

pub fn export_to(path: &str) {
    rawr::Codegen::new()
        .export_type::<structure::Structure>()
        .export_to(path)
        .run()
}
