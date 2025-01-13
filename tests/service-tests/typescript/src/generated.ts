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
export function TestClient(make_request: MakeRequest) {
  return {
    async say_hello(arg: string): Promise<string> {
      return await make_request({
        method: "say_hello",
        payload: [arg],
      });
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
): (request: TestRequest) => Promise<TestResponse> {
  return async (request: TestRequest) => {
    switch (request.method) {
      case "say_hello":
        return {
          method: "say_hello",
          // This handles both sync and async service methods.
          payload: await Promise.resolve(service.say_hello(request.payload[0])),
        };
    }
  };
}
