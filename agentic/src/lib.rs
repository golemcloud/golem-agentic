use crate::agent_registry::AgentId;
use crate::bindings::exports::golem::agent::guest::{AgentRef, StatusUpdate};
use crate::bindings::exports::golem::agent::guest::{AgentType, Guest, GuestAgent};
use crate::bindings::golem::api::host;
use golem_wasm_rpc::WitValue;

pub use type_mapping::*;
pub use agent_construct::*;

pub mod agent;
pub mod agent_instance_registry;
pub mod agent_registry;
pub mod bindings;
mod type_mapping;
mod agent_construct;

#[derive(Clone)]
pub struct ResolvedAgent {
    pub agent: ::std::sync::Arc<dyn agent::Agent + Send + Sync>,
    pub agent_id: String,
}

struct Component;

impl Guest for Component {
    type Agent = ResolvedAgent;

    fn discover_agent_types() -> Vec<AgentType> {
        agent_registry::get_all_agent_definitions()
    }

    fn get_agent(agent_id: String) -> AgentRef {
        let result = agent_registry::get_agent_instance(AgentId(agent_id.clone()));

        if let Some(agent_ref) = result {
            agent_ref
        } else {
            let available_agents = Self::discover_agents()
                .iter()
                .map(|x| x.agent_id.clone())
                .collect::<Vec<_>>()
                .join(", ");

            panic!(
                "Agent with id {} not found. Available agents: {}",
                agent_id, available_agents
            );
        }
    }

    fn discover_agents() -> Vec<AgentRef> {
        let agent_names = agent_registry::get_all_agent_definitions()
            .iter()
            .map(|x| x.type_name.clone())
            .collect::<Vec<_>>();

        let worker_name = host::get_self_metadata().worker_id.worker_name.clone();

        let mut agents = Vec::new();

        for agent_name in agent_names {
            let prefix = format!("{}--{}", worker_name, agent_name);

            agent_registry::get_agent_instances_by_prefix(&prefix)
                .into_iter()
                .for_each(|agent_instance| {
                    agents.push(agent_instance);
                });
        }

        agents
    }
}

impl GuestAgent for ResolvedAgent {
    fn new(agent_type: String, params: Vec<golem_wasm_rpc::WitValue>) -> ResolvedAgent {
        let agent_types = agent_registry::get_all_agent_definitions();

        let agent_type = agent_types
            .iter()
            .find(|x| x.type_name == agent_type)
            .expect(
            format!(
                "Agent definition not found for agent name: {}. Available agents in this app is {}",
                agent_type,
                agent_types
                    .iter()
                    .map(|x| x.type_name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
            .as_str(),
        );

        let agent_initiator = agent_registry::get_agent_initiator(agent_type.type_name.clone());

        if let Some(agent) = agent_initiator {
            let agent = agent.initiate(params);
            agent
        } else {
            panic!(
                "No agent implementation found for agent definition: {}",
                agent_type.type_name
            );
        }
    }

    fn get_id(&self) -> String {
        self.agent_id.clone()
    }

    fn invoke(&self, method_name: String, input: Vec<WitValue>) -> StatusUpdate {
        self.agent.invoke(method_name, input)
    }

    fn get_definition(&self) -> AgentType {
        self.agent.get_definition()
    }
}

bindings::export!(Component with_types_in bindings);
