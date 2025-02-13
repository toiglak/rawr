import WebSocket from "ws";
import { deepEquals } from "bun";
import {
  TestClient,
  type TestRequest,
  type TestResponse,
} from "../../generated";
import type { RawrRequest, RawrResponse } from "rawr";
import type { Structure } from "../../typescript-bindings/schemas/structure";
import type { EnumAdjacentlyTagged } from "../../typescript-bindings/schemas/enumeration";

const addr = process.env.SERVER_ADDR;
if (!addr) throw new Error("SERVER_ADDR not set");
const url = `ws://${addr}`;

// Follows Rust's Structure::default().
const TEST_STRUCTURE: Structure = {
  name: "",
  count: 0,
  is_active: false,
  imported: { value: "" },
  tuple: ["\0", { value: "" }],
  nested_tuple: ["\0", [0, { value: { type: "VariantA" } }]],
  crate_dependency: { value: 0 },
  sequence: [[], [0, 0, 0], [[]]],
  structures: [null, [[], [0, 0, 0], [[]]], [[], [0, 0, 0], [[]]]],
};

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

  // Test complex method.

  const res = await client.complex(TEST_STRUCTURE, 42);
  const expected = { ...TEST_STRUCTURE, count: 42 };
  assert_eq(res, expected);

  // Test sending enum back and forth.
  {
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
