import { type Result } from "../../core/result";
export type ResultsTest<T> = {
  a: Result<string, string>;
  b: Result<[string, string], [number, number]>;
  c: Result<T, string>;
};
