import { ImportedStruct } from '../module';
export type EnumAdjacent =
  | { type: "VariantA" }
  | { type: "VariantB"; data: [] }
  | { type: "VariantC"; data: number }
  | { type: "VariantD"; data: [number, ImportedStruct] }
  | { type: "VariantE"; data: {
  } }
  | { type: "VariantF"; data: {
    a: string;
    b: [number, ImportedStruct];
    c: [string, [number, ImportedStruct]];
  } }
;
