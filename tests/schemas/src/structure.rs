use rawr::Schema;
use schemas_subcrate::StructFromOtherCrate;
use serde::{Deserialize, Serialize};

use crate::{
    array_like::{ArrayLike, TupleStruct},
    module::{nested_module::NestedModuleStruct, ImportedStruct},
};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Schema)]
pub struct Structure {
    pub name: String,
    pub count: i32,
    pub is_active: bool,
    pub imported: ImportedStruct,
    pub tuple: (char, ImportedStruct),
    pub nested_tuple: (char, (i32, NestedModuleStruct)),
    pub crate_dependency: StructFromOtherCrate,
    pub array_like: ArrayLike,
    pub tuple_struct: TupleStruct,
}
