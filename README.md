# golem-agentic

golem-agentic allows you to write AI agents and non-agentic workers in golem with zero knowledge in WIT.

**Designed with AI in mind:** dynamic agent and method discovery, plus plug‑and‑play support for MCP etc are examples. 
In short, the tooling, data types and corresponding APIs to develop AI apps are first‑class citizens.

While dynamic in nature to satisfy AI protocols, users can write **type-safe** code.

This is _**NOT**_ automatic WIT generation based on user code. This implies, we disallow leaking anything about WIT into user's development flow


## Build

```shell
cargo build
```

Refer to [golem-agentic-examples](https://github.com/golemcloud/golem-agentic-examples) repo.
These examples will become templates in golem-cli soon.