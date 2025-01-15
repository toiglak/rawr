import { TestClient, TestServer } from "./generated";

async function main() {
  const handle_request = TestServer({
    async say_hello(arg) {
      await sleep(Math.floor(Math.random() * 1000));
      return `Hello, ${arg}!`;
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

// How would we handle id-s when we send over web sockets?

// as server: we could simply spawn a promise for each request, allowing us to call
// `handle_request`, thus we'd always have a matching response id

// as client: we'd have to disambigue the responses, maybe by using a map of
// promises because web socket responses are not guaranteed to arrive in order, and
// because they come from "one place" (the socket)
