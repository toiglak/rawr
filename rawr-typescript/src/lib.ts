type ReqId = number;

export interface Packet<T> {
  id: ReqId;
  data: T;
}

export type HandleRequest<Req, Res> = (
  req: Packet<Req>
) => Promise<Packet<Res>>;

export type Result<T> = { Ok: T } | { Err: any };

export class RpcClient<Req, Res> {
  private pendingRequests: Map<
    ReqId,
    { resolve: (value: Res) => void; reject: (reason: any) => void }
  > = new Map();
  private nextId: ReqId = 0;

  constructor(private sendRequest: (packet: Packet<Req>) => void) {}

  request(method: string, payload: any): Promise<Res> {
    const id = this.nextId++;
    const packet: Packet<Req> = { id, data: { method, payload } } as any;

    return new Promise((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
      this.sendRequest(packet);
    });
  }

  handleResponse(packet: Packet<Result<Res>>): void {
    const { id, data } = packet;
    const pending = this.pendingRequests.get(id);

    if (!pending) return;

    this.pendingRequests.delete(id);

    if ("Ok" in data) {
      pending.resolve(data.Ok);
    } else {
      pending.reject(data.Err);
    }
  }

  cancelAllPending(reason: any): void {
    for (const { reject } of this.pendingRequests.values()) {
      reject(reason);
    }
    this.pendingRequests.clear();
  }
}
