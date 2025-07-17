pub trait AgentConstruct: Sized {
    fn construct_from_params(params: Vec<golem_wasm_rpc::WitValue>, agent_id: String) -> Self;
}