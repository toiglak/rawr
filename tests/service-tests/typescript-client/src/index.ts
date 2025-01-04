import WebSocket from "ws";
import { type Structure } from "../../typescript-bindings/schemas/structure";
import { deepEquals } from "bun";

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
};

async function checkServer(url: string) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    ws.on("error", reject);
    ws.on("open", () => {
      ws.send(JSON.stringify(TEST_STRUCTURE));
    });
    ws.on("message", (data) => {
      let response = JSON.parse(data.toString());
      assert_eq(response, TEST_STRUCTURE);

      resolve(undefined);
      ws.close();
    });
  });
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
