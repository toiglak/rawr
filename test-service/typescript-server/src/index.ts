const addr = process.env.SERVER_ADDR;
const port = addr && parseInt(addr.split(":")[1]);

Bun.serve({
  port,
  fetch(req, server) {
    // upgrade the request to a WebSocket
    if (server.upgrade(req)) {
      return;
    }
    return new Response("Upgrade failed", { status: 500 });
  },
  websocket: {
    message(ws, message) {
      // echo
      ws.send(message as any);
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
