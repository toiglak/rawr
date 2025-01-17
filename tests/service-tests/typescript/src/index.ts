import type { Structure } from "../../typescript-bindings/schemas/structure";
import { TestClient, TestServer } from "../../generated";

async function main() {
  const handle_request = TestServer({
    say_hello: async function (arg) {
      await sleep(Math.floor(Math.random() * 1000));
      return `Hello, ${arg}!`;
    },
    complex: function (arg: Structure, n: number) {
      arg.count += n;
      return arg;
    },
  });

  const client = TestClient(handle_request);

  // Test ordering (req number should match res number).
  for (let i = 0; i < 10; i++) {
    client.say_hello("World " + i).then((res) => {
      console.log(`[${i++}] ${res}`);
    });
  }
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

main();
