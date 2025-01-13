import { type Response } from "rawr";

// import { type Structure } from "../../typescript-bindings/schemas/structure";
// import { deepEquals } from "bun";
// import { EditorClient, EditorServer } from "./generated";

// async function runTest(url: string) {
//   const handle_request = EditorServer({
//     get_preset: async (hash) => {
//       return {
//         hash,
//         name: "Test Preset",
//         bank: 0,
//         patch: 42,
//         file_path: "hello world /png",
//       };
//     },
//     get_editor_state: async () => {
//       return {
//         active_preset: "hash",
//         flags: { popup_filter_favorite_shown: false },
//         locations: [],
//         favorite_presets: [],
//         presets: [],
//       };
//     },
//     rebuild_preset_cache: async () => {
//       return;
//     },
//     add_favorite: async (hash) => {
//       return;
//     },
//     remove_favorite: async (hash) => {
//       return;
//     },
//     set_flag_popup_filter_favorite_shown: async (shown) => {
//       return;
//     },
//   });

//   const client = EditorClient(handle_request);

//   console.log(await client.get_preset("hash"));
// }

// runTest(url);

// //// UTILITIES

// function assert_eq(expected: any, got: any) {
//   if (!deepEquals(expected, got, true)) {
//     console.error({
//       expected,
//       got,
//     });
//     throw new Error("assertion failed");
//   }
// }
