## Codegen Tests

We're testing code generation with snapshots.

- The `harness` tool generates bindings from `schemas` and checks them against the `snapshots`.
- If something changes, it shows an error. We then can decide if the change is expected.
- If it is, we update the `snapshots`. If not, we fix the bug.
