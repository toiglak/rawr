// import type { Params } from "./plugin.ts";

// import type { Preset } from "./common.ts";
// import type { MakeRequest } from "ezbuf";

// export type EditorMessage =
//   | { tag: "SetPreset"; content: Preset }
//   | { tag: "SetParams"; content: Params }
//   | { tag: "SetDaemonPorts"; content: DaemonPorts };

// export interface DaemonPorts {
//   rpc_port: number;
//   socketio_port: number;
// }

// export type EditorRequest = {
//   method: "push_toast";
//   payload: [string, string, string];
// };

// /**
//  * This function should be supplied to the specific protocol implementation.
//  *
//  * For example, using WebSocket:
//  *
//  * ```ts
//  * import { connect_ws } from "ezbuf";
//  * const ws_client = connect_ws("ws://127.0.0.1:727", EditorClient);
//  * const result = await ws_client.rpc_call();
//  * ```
//  */
// export function EditorClient(make_request: MakeRequest) {
//   return {
//     push_toast(title: string, body: string, level: string): Promise<void> {
//       let request: EditorRequest = {
//         method: "push_toast",
//         payload: [title, body, level],
//       };
//       return make_request(request);
//     },
//   };
// }

// /**
//  * Handles editor requests by delegating to the appropriate service method.
//  *
//  * @param service - An instance of the Editor service.
//  * @returns A function that you should call when handling a client request.
//  */
// export function EditorServer(
//   service: ReturnType<typeof EditorClient>
// ): (request: EditorRequest) => Promise<any> {
//   return async (request: EditorRequest) => {
//     switch (request.method) {
//       case "push_toast":
//         return await service.push_toast(...request.payload);
//     }
//   };
// }
