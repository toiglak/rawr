import type { HandleRequest } from "rawr";
import type { Structure } from "./typescript-bindings/schemas/structure";

export type TestRequest =
  | { method: "say_hello"; payload: [string] }
  | { method: "complex"; payload: [Structure, number] };
export type TestResponse =
  | { method: "say_hello"; payload: string }
  | { method: "complex"; payload: Structure };

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
  make_request: HandleRequest<TestRequest, TestResponse>
) {
  let counter = 0;

  return {
    say_hello: async function (arg: string): Promise<string> {
      const res = await make_request({
        id: counter++,
        data: {
          method: "say_hello",
          payload: [arg],
        },
      });
      if (res.data.method !== "say_hello") {
        throw new Error("Unexpected method: " + res.data.method);
      }
      return res.data.payload;
    },
    complex: async function (arg: Structure, n: number): Promise<Structure> {
      const res = await make_request({
        id: counter++,
        data: {
          method: "complex",
          payload: [arg, n],
        },
      });
      if (res.data.method !== "complex") {
        throw new Error("Unexpected method: " + res.data.method);
      }
      return res.data.payload;
    },
  };
}

export type TestService = {
  // string | Promise<string> is used to allow user to use async or sync functions.
  say_hello: (arg: string) => string | Promise<string>;
  complex: (arg: Structure, n: number) => Structure | Promise<Structure>;
};

/**
 * Handles test requests by delegating to the appropriate service method.
 *
 * @param service - An instance of the Test service.
 * @returns A function that you should call when handling a client request.
 */
export function TestServer(
  service: TestService
): HandleRequest<TestRequest, TestResponse> {
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
      case "complex":
        return {
          id: request.id,
          data: {
            method: "complex",
            payload: await service.complex(
              request.data.payload[0],
              request.data.payload[1]
            ),
          },
        };
    }
  };
}
