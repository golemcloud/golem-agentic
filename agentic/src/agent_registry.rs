use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::binding::exports::golem::agentic::guest::{AgentDefinition, GuestAgent};

static AGENT_DEF_REGISTRY: Lazy<Mutex<Vec<AgentDefinition>>> = Lazy::new(|| Mutex::new(Vec::new()));

// A single component may have multiple agent implementations
static AGENT_IMPL_REGISTRY: Lazy<Mutex<Vec<Box<dyn GuestAgent>>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn register_agent_definition(def: AgentDefinition) {
    AGENT_DEF_REGISTRY.lock().unwrap().push(def);
}

pub fn register_agent_impl(def: Box<dyn GuestAgent>) {
    AGENT_IMPL_REGISTRY.lock().unwrap().push(def);
}


pub fn get_all_agent_definitions() -> Vec<AgentDefinition> {
    AGENT_DEF_REGISTRY.lock().unwrap().clone()
}

pub fn get_all_agent_impls() -> Vec<Box<dyn GuestAgent>> {
    AGENT_IMPL_REGISTRY.lock().unwrap().clone()
}