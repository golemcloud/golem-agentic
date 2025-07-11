use crate::agent_registry::AgentId;
use crate::bindings::exports::golem::agentic_guest::guest::{Agent, AgentRef, StatusUpdate};
use crate::bindings::exports::golem::agentic_guest::guest::{AgentDefinition, Guest, GuestAgent};
use crate::bindings::golem::api::host;

pub mod agent;
pub mod agent_instance_registry;
pub mod agent_registry;
pub mod bindings;

#[derive(Clone)]
pub struct ResolvedAgent {
    pub agent: ::std::sync::Arc<dyn agent::Agent + Send + Sync>,
    pub agent_id: String
}

struct Component;

impl Guest for Component {
    type Agent = ResolvedAgent;

    fn discover_agent_definitions() -> Vec<AgentDefinition> {
        agent_registry::get_all_agent_definitions()
    }

    fn get_agent(agent_id: String) -> AgentRef {
        agent_registry::get_agent_instance(AgentId(agent_id.clone()))
            .expect("Agent with ID {} not found. Available agents in this app are: {}")
    }

    fn discover_agents() -> Vec<AgentRef> {
        let agent_names = agent_registry::get_all_agent_definitions()
            .iter()
            .map(|x| x.agent_name.clone())
            .collect::<Vec<_>>();

        // TODO; this can be improved, currently just trying out
        let worker_name = host::get_self_metadata().worker_id.worker_name.clone();

        let mut agents = Vec::new();

        for agent_name in agent_names {
            let prefix = format!("{}-{}", worker_name, agent_name);

            agent_registry::get_agent_instances_by_prefix(&prefix)
                .into_iter()
                .for_each(|agent_instance| {
                    agents.push(agent_instance);
                });
        }

        agents
    }

    fn create_agent(agent_name: String) -> AgentRef {
        let agent_definitions = agent_registry::get_all_agent_definitions();

        let agent_definition = agent_definitions.iter().find(|x| x.agent_name == agent_name).expect(
            format!("Agent definition not found for agent name: {}. Available agents in this app is {}", agent_name,
                    agent_definitions.iter().map(|x| x.agent_name.clone()).collect::<Vec<_>>().join(", ")).as_str()
        );

        let agent_initiator =
            agent_registry::get_agent_initiator(agent_definition.agent_name.clone());

        if let Some(agent) = agent_initiator {
            let agent = agent.initiate();
            agent
        } else {
            panic!(
                "No agent implementation found for agent definition: {}",
                agent_definition.agent_name
            );
        }
    }
}

impl GuestAgent for ResolvedAgent {
    fn get_agent_id(&self) -> String {
        self.agent_id.clone()
    }

    fn invoke(&self, method_name: String, input: Vec<String>) -> StatusUpdate {
        self.agent.invoke(method_name, input)
    }

    fn get_definition(&self) -> AgentDefinition {
        self.agent.get_definition()
    }
}

bindings::export!(Component with_types_in bindings);
