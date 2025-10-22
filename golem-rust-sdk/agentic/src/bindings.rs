wit_bindgen::generate!({
    path: "../../wit",
    world: "agent-guest",
    generate_all,
    generate_unused_types: true,
    pub_export_macro: true,
    with: {
        "golem:rpc/types@0.2.2": golem_wasm_rpc::golem_rpc_0_2_x::types,
    }
});
