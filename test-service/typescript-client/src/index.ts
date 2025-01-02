import WebSocket from "ws";

const addr = process.env.SERVER_ADDR;
if (!addr) {
  console.error("SERVER_ADDR not set");
  process.exit(1);
}

const url = `ws://${addr}`;

async function checkServer(url: string) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    ws.on("open", () => ws.send("ts_client"));
    ws.on("message", (data) => {
      if (data.toString() === "ts_client") resolve(true);
      else reject(false);
      ws.close();
    });
    ws.on("error", reject);
  });
}

checkServer(url);
