import { type ImportedStruct } from "../module";
export type EnumAdjacentlyTagged =
  | { type: "VariantA" }
  | { type: "VariantB"; data: [] }
  | { type: "VariantC"; data: number }
  | { type: "VariantD"; data: null }
  | { type: "VariantE"; data: ImportedStruct }
  | { type: "VariantF"; data: [number, ImportedStruct] }
  | { type: "VariantG"; data: [number, ImportedStruct] }
  | { type: "VariantH"; data: {
  } }
  | { type: "VariantI"; data: {
    a: number;
    b: ImportedStruct;
  } }
;
