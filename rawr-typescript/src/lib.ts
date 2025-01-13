type ReqId = number;

export interface RawrRequest<Req> {
  id: ReqId;
  data: Req;
}

export interface RawrResponse<Res> {
  id: ReqId;
  data: Res;
}

export type MakeRequest<Req, Res> = (
  data: RawrRequest<Req>
) => Promise<RawrResponse<Res>>;
