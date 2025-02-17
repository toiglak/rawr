use rawr::Schema;
use schemas_subcrate::StructFromOtherCrate;
use serde::{Deserialize, Serialize};

use crate::{
    enumeration::TestEnums,
    module::{nested_module::NestedModuleStruct, ImportedStruct},
    result::ResultsTest,
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
    pub enums: TestEnums,
    pub crate_dependency: StructFromOtherCrate,
    pub sequence: SequenceTypes,
    pub structures: (UnitStruct, NewtypeStruct, TupleStruct),
    pub results: ResultsTest<ImportedStruct>,
}

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct UnitStruct;

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct NewtypeStruct((Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>));

#[derive(Debug, Default, Schema, Serialize, Deserialize, PartialEq)]
pub struct TupleStruct(Vec<String>, [i32; 3], Vec<Vec<ImportedStruct>>);
