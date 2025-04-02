import WebSocket from "ws";
import { deepEquals } from "bun";
import {
  TestClient,
  type TestRequest,
  type TestResponse,
} from "../../manual-codegen";
import { RpcClient } from "rawr-json";
import type { Packet, Result } from "rawr-json";
import type { Structure } from "../../typescript-bindings/schemas/structure";
import type { EnumAdjacentlyTagged } from "../../typescript-bindings/schemas/enumeration";

const addr = process.env.SERVER_ADDR;
if (!addr) throw new Error("SERVER_ADDR not set");
const url = `ws://${addr}`;

const TEST_STRUCTURE: Structure = {
  name: "",
  count: 0,
  is_active: false,
  imported: { value: "" },
  tuple: ["\0", { value: "" }],
  nested_tuple: ["\0", [0, { value: { type: "VariantA" } }]],
  enums: {
    external: "VariantA",
    adjecent: { type: "VariantA" },
  },
  crate_dependency: { value: 0 },
  sequence: [[], [0, 0, 0], [[]]],
  structures: [null, [[], [0, 0, 0], [[]]], [[], [0, 0, 0], [[]]]],
  results: {
    a: { Ok: "Ok" },
    b: { Err: [0, 0] },
    c: { Ok: { value: "" } },
  },
};

async function checkServer(url: string) {
  const ws = new WebSocket(url);

  const rpc = new RpcClient<TestRequest, TestResponse>((packet) => {
    ws.send(JSON.stringify(packet));
  });

  ws.on("message", (data) => {
    const response: Packet<Result<TestResponse>> = JSON.parse(data.toString());
    rpc.handleResponse(response);
  });

  ws.on("close", () => {
    rpc.cancelAllPending(new Error("WebSocket closed"));
  });

  ws.on("error", (err) => {
    rpc.cancelAllPending(err);
  });

  // Wait until we're connected to the server.
  await new Promise((resolve) => ws.on("open", resolve));

  //// Test the service.
  const client = TestClient(rpc);

  // Test async ordering (req number should match res number).
  for (let i = 0; i < 10; i++) {
    try {
      const res = await client.say_hello("World " + i);
      console.log(`[${i}] ${res}`);
    } catch (err) {
      console.error(`Error in say_hello for request ${i}:`, err);
    }
  }

  // Test complex method.
  try {
    const res = await client.complex(TEST_STRUCTURE, 42);
    const expected = { ...TEST_STRUCTURE, count: 42 };
    assert_eq(res, expected);
  } catch (err) {
    console.error("Error in complex method:", err);
  }

  // Test sending enum back and forth.
  try {
    let en: EnumAdjacentlyTagged = { type: "VariantA" };
    let res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantB", data: [] };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantC", data: 42 };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantD", data: null };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantE", data: { value: "string" } };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantF", data: [42, { value: "string" }] };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantG", data: [42, { value: "string" }] };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantH", data: {} };
    res = await client.ping_enum(en);
    assert_eq(res, en);

    en = { type: "VariantI", data: { a: 42, b: { value: "string" } } };
    res = await client.ping_enum(en);
    assert_eq(res, en);
  } catch (err) {
    console.error("Error in ping_enum method:", err);
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
