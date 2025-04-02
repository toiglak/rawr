import { RpcClient } from "rawr-json";
import type { HandleRequest, Result } from "rawr-json";
import type { Structure } from "./typescript-bindings/schemas/structure";
import type { EnumAdjacentlyTagged } from "./typescript-bindings/schemas/enumeration";

export type TestRequest =
  | { method: "say_hello"; payload: [string] }
  | { method: "complex"; payload: [Structure, number] }
  | { method: "ping_enum"; payload: [EnumAdjacentlyTagged] };
export type TestResponse =
  | { method: "say_hello"; payload: string }
  | { method: "complex"; payload: Structure }
  | { method: "ping_enum"; payload: EnumAdjacentlyTagged };

export function TestClient(rpcClient: RpcClient<TestRequest, TestResponse>) {
  return {
    say_hello: async function (arg: string): Promise<string> {
      const result = await rpcClient.request("say_hello", [arg]);
      return result.payload as string;
    },
    complex: async function (arg: Structure, n: number): Promise<Structure> {
      const result = await rpcClient.request("complex", [arg, n]);
      return result.payload as Structure;
    },
    ping_enum: async function (
      arg: EnumAdjacentlyTagged
    ): Promise<EnumAdjacentlyTagged> {
      const result = await rpcClient.request("ping_enum", [arg]);
      return result.payload as EnumAdjacentlyTagged;
    },
  };
}

export type TestService = {
  // string | Promise<string> is used to allow user to use async or sync functions.
  say_hello: (arg: string) => string | Promise<string>;
  complex: (arg: Structure, n: number) => Structure | Promise<Structure>;
  ping_enum: (
    arg: EnumAdjacentlyTagged
  ) => EnumAdjacentlyTagged | Promise<EnumAdjacentlyTagged>;
};

export function TestServer(
  service: TestService
): HandleRequest<TestRequest, Result<TestResponse>> {
  return async (request) => {
    try {
      switch (request.data.method) {
        case "say_hello":
          return {
            id: request.id,
            data: {
              Ok: {
                method: "say_hello",
                payload: await service.say_hello(request.data.payload[0]),
              },
            },
          };
        case "complex":
          return {
            id: request.id,
            data: {
              Ok: {
                method: "complex",
                payload: await service.complex(
                  request.data.payload[0],
                  request.data.payload[1]
                ),
              },
            },
          };
        case "ping_enum":
          return {
            id: request.id,
            data: {
              Ok: {
                method: "ping_enum",
                payload: await service.ping_enum(request.data.payload[0]),
              },
            },
          };
      }
    } catch (error) {
      // TODO: I don't think catching all errors is a good idea. I think we should
      // just throw and let developer fix the logic bug.
      return {
        id: request.id,
        data: {
          Err:
            error instanceof Error
              ? `TestServer handler threw: ${error.message}`
              : "TestServer handler threw: Unknown error",
        },
      };
    }
  };
}
