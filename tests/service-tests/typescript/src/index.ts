import { TestClient, TestServer } from "./generated";

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function runTest() {
  let counter = 0;

  const handle_request = TestServer({
    async say_hello(arg) {
      await sleep(Math.floor(Math.random() * 1000));
      return `Hello, ${arg}!`;
    },
  });

  const client = TestClient(handle_request);

  // Currently fails:
  // Test ordering (req should match res).
  for (let i = 0; i < 10; i++) {
    client.say_hello("World " + i).then((res) => {
      console.log(`[${counter++}] ${res}`);
    });
  }
}

runTest();
