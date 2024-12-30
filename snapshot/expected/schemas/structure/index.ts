import { ImportedStruct } from '../module';
export type MyData = {
  name: string;
  count: number;
  is_active: boolean;
  imported: ImportedStruct;
  tuple: [string, ImportedStruct];
  nested_tuple: [string, [number, ImportedStruct]];
};
