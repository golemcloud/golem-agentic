use crate::agent_instance_registry::AgentName;
use crate::bindings::exports::golem::agent::guest::AgentType;
use crate::bindings::golem::agent::common::AgentConstructor;
use crate::bindings::golem::agent::common::AgentDependency;
use crate::bindings::golem::agent::common::AgentMethod;
use crate::bindings::golem::agent::common::DataValue;
use crate::ResolvedAgent;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type AgentTypeName = String;

#[derive(Hash, PartialEq, Eq)]
pub struct AgentId(pub String);

// An agent-type which is devoid of a few details from what's in WIT

static CONSTRUCTOR_REGISTRY: once_cell::sync::Lazy<
    std::sync::Mutex<HashMap<String, Vec<(String, String)>>>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

pub fn register_constructor(agent_type_name: String, constructor: Vec<(String, String)>) {
    CONSTRUCTOR_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_type_name, constructor);
}

pub fn get_constructor(agent_type_name: &str) -> Option<Vec<(String, String)>> {
    CONSTRUCTOR_REGISTRY
        .lock()
        .unwrap()
        .get(agent_type_name)
        .cloned()
}

#[derive(Clone, Debug)]
pub struct GenericAgentType {
    pub type_name: String,
    pub description: String,
    pub methods: Vec<AgentMethod>,
    pub requires: Vec<AgentDependency>,
}

impl GenericAgentType {
    pub fn to_agent_type(&self, agent_constructor: AgentConstructor) -> AgentType {
        AgentType {
            type_name: self.type_name.clone(),
            description: self.description.clone(),
            constructor: agent_constructor,
            methods: self.methods.clone(),
            dependencies: self.requires.clone(),
        }
    }
}

pub struct AgentRefInternal {
    resolved_agent: ResolvedAgent,
    agent_name: String,
}

static GENERIC_AGENT_TYPE_REGISTRY: Lazy<Mutex<HashMap<AgentTypeName, GenericAgentType>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static AGENT_TYPE_REGISTRY: Lazy<Mutex<HashMap<AgentTypeName, AgentType>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Given an agent name, we can register an impl of agent-initiator
// This helps with initiating an agent given an agent name
static AGENT_INITIATOR_REGISTRY: Lazy<
    Mutex<HashMap<AgentName, Arc<dyn AgentInitiator + Send + Sync>>>,
> = Lazy::new(|| Mutex::new(HashMap::new()));

static AGENT_INSTANCE_REGISTRY: Lazy<Mutex<Option<AgentRefInternal>>> =
    Lazy::new(|| Mutex::new(None));

pub fn register_agent_definition(agent_trait_name: String, def: AgentType) {
    AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_trait_name, def);
}

pub fn register_generic_agent_type(agent_trait_name: String, def: GenericAgentType) {
    GENERIC_AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_trait_name, def);
}

pub fn register_agent_type(agent_trait_name: String, def: AgentType) {
    AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_trait_name, def);
}

pub fn register_agent_initiator(
    agent_id: AgentName,
    implementation: Arc<dyn AgentInitiator + Send + Sync>,
) {
    AGENT_INITIATOR_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_id, implementation);
}

pub fn register_agent_instance(
    agent_id: AgentId,
    agent_name: String,
    resolved_agent: ResolvedAgent,
) {
    let mut registry = AGENT_INSTANCE_REGISTRY.lock().unwrap();

    *registry = Some(AgentRefInternal {
        resolved_agent,
        agent_name,
    });
}

pub fn get_agent_instance() -> Option<ResolvedAgent> {
    AGENT_INSTANCE_REGISTRY
        .lock()
        .unwrap()
        .as_ref()
        .map(|agent_ref| agent_ref.resolved_agent.clone())
}

pub fn get_agent_def_by_name(agent_trait_name: &str) -> Option<AgentType> {
    AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .get(agent_trait_name)
        .cloned()
}

pub fn get_generic_agent_type_by_name(agent_trait_name: &str) -> Option<GenericAgentType> {
    GENERIC_AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .get(agent_trait_name)
        .cloned()
}

pub fn get_all_generic_agent_types() -> Vec<GenericAgentType> {
    GENERIC_AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn get_all_agent_definitions() -> Vec<AgentType> {
    AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .values()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn get_agent_initiator(
    agent_trait_name: AgentTypeName,
) -> Option<Arc<dyn AgentInitiator + Send + Sync>> {
    AGENT_INITIATOR_REGISTRY
        .lock()
        .unwrap()
        .get(&agent_trait_name)
        .cloned()
}

pub trait AgentInitiator: Send + Sync {
    fn initiate(&self, params: DataValue) -> ResolvedAgent;
}
