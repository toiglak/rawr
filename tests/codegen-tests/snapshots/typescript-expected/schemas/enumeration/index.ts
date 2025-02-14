import { type ImportedStruct } from "../module";
export type EnumAdjacentlyTagged =
  | { type: "VariantA" }
  | { type: "VariantB"; data: [] }
  | { type: "VariantC"; data: number }
  | { type: "VariantD"; data: null }
  | { type: "VariantE"; data: ImportedStruct }
  | { type: "VariantF"; data: [number, ImportedStruct] }
  | { type: "VariantG"; data: [number, ImportedStruct] }
  | { type: "VariantH"; data: {  } }
  | { type: "VariantI"; data: { a: number, b: ImportedStruct } }
;
export type EnumExternallyTagged =
  | "VariantA"
  | { "VariantB": [] }
  | { "VariantC": number }
  | { "VariantD": null }
  | { "VariantE": ImportedStruct }
  | { "VariantF": [number, ImportedStruct] }
  | { "VariantG": [number, ImportedStruct] }
  | { "VariantH": {  } }
  | { "VariantI": { a: number, b: ImportedStruct } }
;
export type TestEnums = {
  external: EnumExternallyTagged;
  adjecent: EnumAdjacentlyTagged;
};
