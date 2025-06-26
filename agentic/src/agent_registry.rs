use crate::agent::Agent;
use crate::binding::exports::golem::agentic::guest::{AgentDefinition, GuestAgent};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type AgentTraitName = String;

static AGENT_DEF_REGISTRY: Lazy<Mutex<HashMap<AgentTraitName, AgentDefinition>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// A single component may have multiple agent implementations
static AGENT_IMPL_REGISTRY: Lazy<Mutex<Vec<Arc<dyn Agent + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(Vec::new()));

pub fn register_agent_definition(agent_trait_name: String, def: AgentDefinition) {
    AGENT_DEF_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_trait_name, def);
}

pub fn register_agent_impl(def: Arc<dyn Agent + Send + Sync>) {
    AGENT_IMPL_REGISTRY.lock().unwrap().push(def);
}

pub fn get_agent_def_by_name(agent_trait_name: &str) -> Option<AgentDefinition> {
    AGENT_DEF_REGISTRY
        .lock()
        .unwrap()
        .get(agent_trait_name)
        .cloned()
}

pub fn get_all_agent_definitions() -> Vec<AgentDefinition> {
    AGENT_DEF_REGISTRY
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn get_all_agent_impls() -> Vec<Arc<dyn Agent + Send + Sync>> {
    AGENT_IMPL_REGISTRY.lock().unwrap().clone()
}
