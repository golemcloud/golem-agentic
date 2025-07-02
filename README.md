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

## Test

Currently testing is done using low level APIs through Rib. This will get better soon

![img.png](img.png)


## TODO (to be made as tickets)
**Step 1:** Currently tested with just string arguments,. Make sure paramerter types and return types talks proper WitType , and implement proper mapping between WitType and argument types.
**Step 2:** Make sure external interactions look like typesafe rpc calls when interactiing with agents - meaning golem repl shouldn't talk low level WITs. This should be possible if typesafe rpc is possible. Multiple suggestions exist such as custom_seciton which looks dead simple
**Step 3: **Make sure code_first_agent branch in golem OSS is merged into OSS. golem-wit changes and implementation already exist here. Do it properly though
**Step 4:** Make sure agents can exist in different modules within a workspace and can exist as different workers.
**Step 5: **Integrate with Golem CLI and template these examples
**Step 6: **Incrementally add AI specific types into the root WIT. We can break this into multiple tickets.
