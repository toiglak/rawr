#[linkme::distributed_slice]
pub static REGISTRY: [&'static str];

pub use linkme::*;

#[linkme::distributed_slice]
pub static REGISTR2Y: [fn() -> TypeSchema];

struct TypeSchema {}
