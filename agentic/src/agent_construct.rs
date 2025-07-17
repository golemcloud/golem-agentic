use golem_wasm_rpc::WitType;

pub trait AgentConstruct: Sized {
    fn construct_from_params(params: Vec<golem_wasm_rpc::WitValue>, agent_id: String) -> Self;
    fn get_params() -> Vec<(String, WitType)>;
    fn get_agent_dependencies() -> Vec<String>;
}
