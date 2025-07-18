use crate::agent_instance_registry::AgentName;
use crate::bindings::exports::golem::agent::guest::{AgentRef, AgentType, WitValue};
use crate::ResolvedAgent;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::bindings::golem::agent::common::{AgentDependency, AgentMethod, AgentConstructor};

type AgentTypeName = String;

#[derive(Hash, PartialEq, Eq)]
pub struct AgentId(pub String);

// An agent-type which is devoid of a few details from what's in WIT

static CONSTRUCTOR_REGISTRY: once_cell::sync::Lazy<std::sync::Mutex<HashMap<String, Vec<(String, String)>>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(HashMap::new()));

pub fn register_constructor(
    agent_type_name: String,
    constructor: Vec<(String, String)>,
) {
    CONSTRUCTOR_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_type_name, constructor);
}

pub fn get_constructor(
    agent_type_name: &str,
) -> Option<Vec<(String, String)>> {
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
    pub requires: Vec<AgentDependency>
}

impl GenericAgentType {
    pub fn to_agent_type(&self, agent_constructor: AgentConstructor) -> AgentType {
        AgentType {
            type_name: self.type_name.clone(),
            description: self.description.clone(),
            agent_constructor: agent_constructor,
            methods: self.methods.clone(),
            requires: self.requires.clone(),
        }
    }
}

pub struct AgentRefInternal {
    inner_instance: crate::bindings::exports::golem::agent::guest::Agent,
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

// Once the agent is initiated, we can register the agent instance for quick lookups
static AGENT_INSTANCE_REGISTRY: Lazy<Mutex<HashMap<AgentId, AgentRefInternal>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn register_agent_definition(agent_trait_name: String, def: AgentType) {
    AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_trait_name, def);
}

pub fn register_generic_agent_type(
    agent_trait_name: String,
    def: GenericAgentType,
) {
    GENERIC_AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .insert(agent_trait_name, def);
}

pub fn register_agent_type(
    agent_trait_name: String,
    def: AgentType,
) {
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

//Get all agent instances for a given string which is a prefix of agent_id
// This is not great, but I wanted to test
pub fn get_agent_instances_by_prefix(
    prefix: &str,
) -> Vec<crate::bindings::exports::golem::agent::guest::AgentRef> {
    AGENT_INSTANCE_REGISTRY
        .lock()
        .unwrap()
        .iter()
        .filter_map(|(agent_id, agent_ref_internal)| {
            if agent_id.0.starts_with(prefix) {
                Some(AgentRef {
                    agent_id: agent_ref_internal.resolved_agent.agent_id.clone(),
                    agent_name: agent_ref_internal.agent_name.clone(),
                    agent_handle: agent_ref_internal.inner_instance.handle(),
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn register_agent_instance(
    agent_id: AgentId,
    agent_name: String,
    agent_instance: crate::bindings::exports::golem::agent::guest::Agent,
    resolved_agent: ResolvedAgent,
) {
    AGENT_INSTANCE_REGISTRY.lock().unwrap().insert(
        agent_id,
        AgentRefInternal {
            inner_instance: agent_instance,
            resolved_agent,
            agent_name,
        },
    );
}

pub fn get_agent_instance(agent_id: AgentId) -> Option<AgentRef> {
    AGENT_INSTANCE_REGISTRY
        .lock()
        .unwrap()
        .get(&agent_id)
        .map(|agent_ref_internal| AgentRef {
            agent_id: agent_id.0.clone(),
            agent_name: agent_ref_internal.agent_name.clone(),
            agent_handle: agent_ref_internal.inner_instance.handle(),
        })
}

pub fn get_agent_def_by_name(agent_trait_name: &str) -> Option<AgentType> {
    AGENT_TYPE_REGISTRY
        .lock()
        .unwrap()
        .get(agent_trait_name)
        .cloned()
}

pub fn get_generic_agent_type_by_name(
    agent_trait_name: &str,
) -> Option<GenericAgentType> {
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
    fn initiate(&self, params: Vec<WitValue>) -> ResolvedAgent;
}
