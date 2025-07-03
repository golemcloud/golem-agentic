wit_bindgen::generate!({
    path: "wit",
    world: "agentic-guest",
    generate_all,
    generate_unused_types: true,
    pub_export_macro: true,
    with: {
         "golem:rpc/types@0.2.1": golem_wasm_rpc::golem_rpc_0_2_x::types,
    }
});
