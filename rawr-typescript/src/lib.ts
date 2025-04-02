type ReqId = number;

export interface Packet<T> {
  id: ReqId;
  data: T;
}

export type HandleRequest<Req, Res> = (
  req: Packet<Req>
) => Promise<Packet<Res>>;
