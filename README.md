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


## TODO (to be made as tickets)

Mostly the prototype implementation already did the complex or confusing parts, and what's remaining are details.

* Currently tested with just string arguments,. Make sure paramerter types and return types talks proper WitType , and implement proper mapping between WitType and argument types.
* Make sure external interactions look like typesafe rpc calls when interactiing with agents - meaning golem repl shouldn't talk low level WITs. This is kind of in progress, but tere were a few time-consuming things to get it going properly. This should be easily possible in a couple of days since RPC is already proven to be working with typesafe interactions.
* Make sure [code_first_agent](https://github.com/golemcloud/golem/compare/main...code_first_agent) branch in golem OSS is merged into OSS. golem-wit changes and implementation already exist here. Some integration tests already exist in this branch.
* Make sure agents can exist in different modules within a workspace and can exist as different workers. Thuis will change the self-metadata querying within the OSS implementation done now, and will end up making use discover_all_agents within host.
* Async vs sync calls implementations and examples. Example: If an agent's implementation depends on a remote call to another agent's function (which is not async) it is still in async since it's a remote call. Does this mean we recommend users to always have `async` functions in their agents to reduce complexit? What about blocking-invoke vs fire-and-forget that exist in wasm-rpc? Probably we need parallels within code-first-agents too.
* Integrate with Golem CLI and template these examples
* Incrementally add AI specific types into the root WIT. We can break this into multiple tickets.
