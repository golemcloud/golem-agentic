use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::binding::exports::golem::agentic::guest::AgentDefinition;

static REGISTRY: Lazy<Mutex<Vec<AgentDefinition>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn register_agent_definition(def: AgentDefinition) {
    REGISTRY.lock().unwrap().push(def);
}

pub fn get_all_agent_definitions() -> Vec<AgentDefinition> {
    REGISTRY.lock().unwrap().clone()
}