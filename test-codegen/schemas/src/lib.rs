pub mod enumeration;
pub mod module;
pub mod structure;

pub fn import() {
    // Dummy function to force the compiler to include static variables from this
    // crate in the final binary, thus allowing the types marked for export to be
    // registered.

    // It seems like this function gets optimized out when running in release mode.
    // This fixes that problem.
    std::hint::black_box(());
}
