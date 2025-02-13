import { type ImportedStruct } from "../module";
import { type NestedModuleStruct } from "../module/nested_module";
import { type SequenceTypes } from "../sequence";
import { type StructFromOtherCrate } from "../../schemas_subcrate";
export type NewtypeStruct = [string[], number[], ImportedStruct[][]];
export type Structure = {
  name: string;
  count: number;
  is_active: boolean;
  imported: ImportedStruct;
  tuple: [string, ImportedStruct];
  nested_tuple: [string, [number, NestedModuleStruct]];
  crate_dependency: StructFromOtherCrate;
  sequence: SequenceTypes;
  structures: [UnitStruct, NewtypeStruct, TupleStruct];
};
export type TupleStruct = [string[], number[], ImportedStruct[][]];
export type UnitStruct = null;
