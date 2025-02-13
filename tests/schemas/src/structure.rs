use rawr::Schema;
use schemas_subcrate::StructFromOtherCrate;
use serde::{Deserialize, Serialize};

use crate::{
    module::{nested_module::NestedModuleStruct, ImportedStruct},
    sequence::SequenceTypes,
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
    pub sequence: SequenceTypes,
    pub structures: (UnitStruct, NewtypeStruct, TupleStruct),
}

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct UnitStruct;

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct NewtypeStruct((Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>));

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct TupleStruct(Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>);
