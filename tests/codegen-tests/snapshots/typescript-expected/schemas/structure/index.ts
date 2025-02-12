import { type ImportedStruct } from "../module";
import { type NestedModuleStruct } from "../module/nested_module";
import { type StructFromOtherCrate } from "../../schemas_subcrate";
export type Structure = {
  name: string;
  count: number;
  is_active: boolean;
  imported: ImportedStruct;
  tuple: [string, ImportedStruct];
  nested_tuple: [string, [number, NestedModuleStruct]];
  crate_dependency: StructFromOtherCrate;
  array_like: [string[], number[], number[][]];
};
