import type { Structure } from "../../typescript-bindings/schemas/structure";
import {
  TestClient,
  TestServer,
  type TestRequest,
  type TestResponse,
} from "../../manual-codegen";
import { RpcClient } from "rawr-json";

async function main() {
  const handleRequest = TestServer({
    async say_hello(arg) {
      await sleep(Math.floor(Math.random() * 1000));
      return `Hello, ${arg}!`;
    },
    complex(arg: Structure, n: number) {
      arg.count += n;
      return arg;
    },
    ping_enum(arg) {
      return arg;
    },
  });

  const rpc = new RpcClient<TestRequest, TestResponse>((packet) => {
    handleRequest(packet).then((res) => {
      rpc.handleResponse(res);
    });
  });

  const client = TestClient(rpc);

  // Test ordering (req number should match res number).
  for (let i = 0; i < 10; i++) {
    client.say_hello("World " + i).then((res) => {
      if (res !== `Hello, World ${i}!`) {
        throw new Error(`Expected "Hello, World ${i}!", but got "${res}"`);
      }
    });
  }
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main();
