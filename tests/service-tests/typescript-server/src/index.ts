import type { RawrRequest, RawrResponse } from "rawr";
import {
  TestServer,
  type TestRequest,
  type TestResponse,
} from "../../generated";
import type { Structure } from "../../typescript-bindings/schemas/structure";

const addr = process.env.SERVER_ADDR;
const port = addr && parseInt(addr.split(":")[1]);

const handle_request = TestServer({
  say_hello: function (arg) {
    return `Hello, ${arg}!`;
  },
  complex: function (arg: Structure, n: number) {
    arg.count += n;
    return arg;
  },
  ping_enum: function (arg) {
    return arg;
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
