export type Result<T, E> =
  | { "Ok": T }
  | { "Err": E }
;
