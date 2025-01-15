import type { RawrRequest, RawrResponse } from "rawr";
import { TestServer, type TestRequest, type TestResponse } from "./generated";

const addr = process.env.SERVER_ADDR;
const port = addr && parseInt(addr.split(":")[1]);

const handle_request = TestServer({
  async say_hello(arg) {
    return `Hello, ${arg}!`;
  },
});

Bun.serve({
  port,
  fetch(req, server) {
    if (server.upgrade(req)) return;
    return new Response("Upgrade to websocket failed", { status: 500 });
  },
  websocket: {
    async message(ws, message) {
      const req: RawrRequest<TestRequest> = JSON.parse(message as any);
      const res: RawrResponse<TestResponse> = await handle_request(req);
      ws.send(JSON.stringify(res));
    },
    open(ws) {
      console.log("WebSocket connection opened");
    },
    close(ws, code, message) {
      console.log(`WebSocket connection closed: ${code} ${message}`);
    },
    drain(ws) {
      console.log("WebSocket is ready to receive more data");
    },
  },
});
