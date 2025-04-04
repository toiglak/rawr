import { task as herebyTask } from "hereby";
import { createServer } from "net";
import { $ } from "execa";

// If there are any long running processes that need to be killed at the end of the
// test, push them to this array.
let processes = [];
// List of server addresses to test clients against.
let serverAddresses = [];

export const GENERATE_BINDINGS = task({
  name: "generate-bindings",
  hiddenFromTaskList: true,
  async run() {
    await $`cargo run --bin service-tests-run-codegen`;
  },
});

export const BUILD_RUST_SERVER = task({
  name: "build-rust-server",
  hiddenFromTaskList: true,
  async run() {
    await $`cargo build --bin rust_server`;
  },
});

export const START_RUST_SERVER = task({
  name: "start-rust-server",
  hiddenFromTaskList: true,
  dependencies: [BUILD_RUST_SERVER],
  async run() {
    const addr = await getFreeLocalAddress();
    serverAddresses.push(addr);

    // Start the server as a background process
    const serverProcess = $({
      env: { SERVER_ADDR: addr },
      reject: false,
    })`cargo run --bin rust_server`;
    processes.push(serverProcess);

    // Wait for server to be ready
    await new Promise((r) => setTimeout(r, 100));

    return addr;
  },
});

export const START_TS_SERVER = task({
  name: "start-typescript-server",
  hiddenFromTaskList: true,
  dependencies: [GENERATE_BINDINGS],
  async run() {
    const cwd = "./tests/service-tests/typescript-server";

    // Type check.
    await $`bun run tsc --project ${cwd}/tsconfig.json`;

    const addr = await getFreeLocalAddress();
    serverAddresses.push(addr);

    const serverProcess = $({
      cwd: cwd,
      env: { SERVER_ADDR: addr },
    })`bun src/index.ts`;
    processes.push(serverProcess);

    // Wait for server to be ready
    await new Promise((r) => setTimeout(r, 100));

    return addr;
  },
});

export const RUN_RUST_CLIENT = task({
  name: "run-rust-client",
  hiddenFromTaskList: true,
  dependencies: [START_TS_SERVER, START_RUST_SERVER],
  async run() {
    for (const addr of serverAddresses) {
      await $({
        env: { SERVER_ADDR: addr },
      })`cargo run --bin rust_client`;
    }
  },
});

export const RUN_TS_CLIENT = task({
  name: "run-typescript-client",
  hiddenFromTaskList: true,
  dependencies: [GENERATE_BINDINGS, START_TS_SERVER, START_RUST_SERVER],
  async run() {
    const cwd = "./tests/service-tests/typescript-client";

    // Type check
    await $`bun run tsc --project ${cwd}/tsconfig.json`;

    // Run client against all servers
    for (const addr of serverAddresses) {
      await $({
        cwd: cwd,
        env: { SERVER_ADDR: addr },
      })`bun src/index.ts`;
    }
  },
});

export const RUN_ALL_TESTS = task({
  name: "test:all",
  dependencies: [RUN_RUST_CLIENT, RUN_TS_CLIENT],
  async run() {
    // Other tasks run, we can clean up.
    await quit();
  },
});

// Default task
export default RUN_ALL_TESTS;

//// Utils

async function getFreeLocalAddress() {
  return new Promise((resolve) => {
    const server = createServer();
    server.listen(0, () => {
      const address = server.address();
      const port = address.port;
      server.close(() => {
        resolve(`127.0.0.1:${port}`);
      });
    });
  });
}

//// Handle process termination

/** @param {import("hereby").TaskOptions} config */
// Wrap hereby task to terminate long running processes on throw.
function task(config) {
  const run = config.run;

  config.run = async () => {
    try {
      return await run();
    } catch (error) {
      await quit();
      throw error;
    }
  };

  return herebyTask(config);
}

// Register cleanup handler for unexpected terminations
process.on("SIGINT", async () => {
  await quit();
});

async function quit() {
  const terminations = processes.map(async (proc) => {
    // Skip already terminated processes
    if (proc.killed) return;

    // Send SIGTERM to the process
    proc.kill("SIGTERM");

    try {
      // Wait for it to exit
      await proc;
    } catch (err) {
      if (err.signal === "SIGTERM" || err.isCanceled) {
        // Ignore expected termination errors
        return;
      }
      console.error("Unexpected error terminating process:", err);
      process.exit(1);
    }
  });

  await Promise.all(terminations);
}
