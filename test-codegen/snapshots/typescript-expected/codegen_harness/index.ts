import { ImportedStruct } from '../schemas/module';
export type MainStruct = {
  a: number;
  b: ImportedStruct;
};
