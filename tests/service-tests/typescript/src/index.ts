import { TestClient, TestServer } from "./generated";

async function runTest() {
  const handle_request = TestServer({
    say_hello(arg) {
      return `Hello, ${arg}!`;
    },
  });

  const client = TestClient(handle_request);

  console.log(await client.say_hello("hash"));
}

runTest();
