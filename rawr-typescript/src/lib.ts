type ReqId = number;
type Waker = (data: any) => void;

export interface Request {
  id: ReqId;
  data: string;
}

export interface Response {
  id: ReqId;
  data: string;
}

/**
 * Client implementation to create.
 *
 * This function is automatically generated from a service by ezbuf.
 */
export type CreateClient<C> = (make_request: MakeRequest) => C;

/** TODO: This should be generic */
export type RequestData = any;
export type MakeRequest = (data?: RequestData) => Promise<any>;

/**
 * Connect to a websocket server.
 *
 * @param url The websocket url to connect to.
 * @param create_client The client implementation to create.
 * @returns The client instance.
 */
export function connect_ws<C>(url: string, create_client: CreateClient<C>): C {
  let id = 0;
  const ws = new WebSocket(url);
  const request_map: Map<ReqId, Waker> = new Map();

  const make_request = async (data?: any): Promise<any> => {
    // TODO: Just await ws.open() before returning the client.
    if (ws.readyState != ws.OPEN) {
      await new Promise((resolve) => {
        ws.onopen = resolve;
      });
    }

    const request: Request = {
      id: ++id,
      data: JSON.stringify(data),
    };

    ws.send(JSON.stringify(request));
    return new Promise((resolve) => {
      request_map.set(id, resolve);
    });
  };

  const handle_response = (response: Response) => {
    const waker = request_map.get(response.id);
    if (waker) {
      waker(JSON.parse(response.data));
    }
  };

  ws.onerror = (err) => {
    console.error(err);
  };

  ws.onmessage = (response) => {
    handle_response(JSON.parse(response.data));
  };

  ws.onclose = (a) => {
    console.log(`Disconnected (${a})`);
  };

  return create_client(make_request);
}

/**
 * Request handler for ipc.
 *
 * Your implementation should forward the request to the server and call `resolve` with
 * the response from it. Do note that `connect_ipc` resolves call `id`, so you don't need
 * to worry about resolving requests in order.
 *
 */
type HandleRequest = (
  request: Request,
  resolve: (response: Response) => void
) => void;

/**
 * Connect to rpc server through custom protocol.
 *
 * @param handle_request The request handler.
 * @param create_client The client implementation to create.
 * @returns The client instance.
 */
export function connect_custom<C>(
  handle_request: HandleRequest,
  create_client: CreateClient<C>
): C {
  let id = 0;
  const request_map: Map<ReqId, Waker> = new Map();

  const handle_response = (response: Response) => {
    const waker = request_map.get(response.id);
    if (waker) {
      waker(JSON.parse(response.data));
    }
  };

  const make_request = async (data?: any): Promise<any> => {
    const request: Request = {
      id: ++id,
      data: JSON.stringify(data),
    };
    return new Promise((resolve) => {
      request_map.set(id, resolve);
      handle_request(request, handle_response);
    });
  };

  return create_client(make_request);
}
