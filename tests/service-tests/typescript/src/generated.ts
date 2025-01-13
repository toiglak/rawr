import type { MakeRequest } from "rawr";

export type TestRequest = { method: "say_hello"; payload: [string] };
export type TestResponse = { method: "say_hello"; payload: string };

/**
 * This function should be supplied to the specific protocol implementation.
 *
 * For example, using WebSocket:
 *
 * ```ts
 * import { connect_ws } from "ezbuf";
 * const ws_client = connect_ws("ws://127.0.0.1:727", TestClient);
 * const result = await ws_client.rpc_call();
 * ```
 */
export function TestClient(
  make_request: MakeRequest<TestRequest, TestResponse>
) {
  return {
    async say_hello(arg: string): Promise<string> {
      const res = await make_request({
        id: 0,
        data: {
          method: "say_hello",
          payload: [arg],
        },
      });
      return res.data.payload;
    },
  };
}

export type TestService = {
  // string | Promise<string> is used to allow user to use async or sync functions.
  say_hello: (arg: string) => string | Promise<string>;
};

/**
 * Handles test requests by delegating to the appropriate service method.
 *
 * @param service - An instance of the Test service.
 * @returns A function that you should call when handling a client request.
 */
export function TestServer(
  service: TestService
): MakeRequest<TestRequest, TestResponse> {
  return async (request) => {
    switch (request.data.method) {
      case "say_hello":
        return {
          id: request.id,
          data: {
            method: "say_hello",
            payload: await service.say_hello(request.data.payload[0]),
          },
        };
    }
  };
}
