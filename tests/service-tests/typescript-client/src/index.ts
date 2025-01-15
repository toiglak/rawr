import WebSocket from "ws";
import { deepEquals } from "bun";
import { TestClient, type TestRequest, type TestResponse } from "./generated";
import type { RawrRequest, RawrResponse } from "rawr";

const addr = process.env.SERVER_ADDR;
if (!addr) throw new Error("SERVER_ADDR not set");
const url = `ws://${addr}`;

async function checkServer(url: string) {
  const ws = new WebSocket(url);
  const resMap = new Map<number, (res: RawrResponse<TestResponse>) => void>();

  // ws.on("error", reject);
  ws.on("message", (data) => {
    const response: RawrResponse<TestResponse> = JSON.parse(data.toString());
    const resolve = resMap.get(response.id);
    if (resolve) resMap.delete(response.id);
    if (resolve) resolve(response);
  });

  async function handle_request(
    request: RawrRequest<TestRequest>
  ): Promise<RawrResponse<TestResponse>> {
    return new Promise((resolve) => {
      resMap.set(request.id, resolve);
      ws.send(JSON.stringify(request));
    });
  }

  // Wait until we're connected to the server.
  await new Promise((resolve) => ws.on("open", resolve));

  //// Test the service.

  const client = TestClient(handle_request);

  // TODO: Test async ordering (req number should match res number).
  for (let i = 0; i < 10; i++) {
    // client.say_hello("World " + i).then((res) => {
    //   console.log(`[${i++}] ${res}`);
    // });
    const res = await client.say_hello("World " + i);
    console.log(`[${i++}] ${res}`);
  }

  ws.close();
}

checkServer(url);

//// UTILITIES

function assert_eq(expected: any, got: any) {
  if (!deepEquals(expected, got, true)) {
    console.error({
      expected,
      got,
    });
    throw new Error("assertion failed");
  }
}
