use crate::bindings::exports::golem::agentic_guest::guest::{Agent, StatusUpdate};
use crate::bindings::exports::golem::agentic_guest::guest::{AgentDefinition, Guest, GuestAgent};

pub mod agent;
pub mod agent_instance_counter;
pub mod agent_registry;
pub mod bindings;

pub struct ResolvedAgent {
    pub agent: ::std::sync::Arc<dyn agent::Agent + Send + Sync>,
}

struct Component;

impl Guest for Component {
    type Agent = ResolvedAgent;

    fn discover_agent_definitions() -> Vec<AgentDefinition> {
        agent_registry::get_all_agent_definitions()
    }

    fn create_agent(agent_name: String) -> Agent {
        let agent_definitions = agent_registry::get_all_agent_definitions();

        let agent_definition = agent_definitions.iter().find(|x| x.agent_name == agent_name).expect(
            format!("Agent definition not found for agent name: {}. Available agents in this app is {}", agent_name,
                agent_definitions.iter().map(|x| x.agent_name.clone()).collect::<Vec<_>>().join(", ")).as_str()
        );

        let agent_impl_resolver =
            agent_registry::get_agent_impl_by_def(agent_definition.agent_name.clone());

        if let Some(resolver) = agent_impl_resolver {
            let agent = resolver.resolve_agent_impl();
            Agent::new(ResolvedAgent { agent })
        } else {
            panic!(
                "No agent implementation found for agent definition: {}",
                agent_definition.agent_name
            );
        }
    }

    fn get_agent(agent_id: String) -> bindings::exports::golem::agentic_guest::guest::Agent {
        let agent_names = agent_registry::get_all_agent_definitions()
            .iter()
            .map(|x| x.agent_name.clone())
            .collect::<Vec<_>>();

        let mut agent = None;

        for agent_name in agent_names.iter() {
            let resolver = agent_registry::get_agent_impl_by_def(agent_name.clone()).unwrap();
            let resolved_agent = resolver.resolve_agent_impl();
            if resolved_agent.agent_id() == agent_id {
                agent = Some(Agent::new(ResolvedAgent { agent: resolved_agent }));
                break;
            }
        }

        agent.expect(format!(
            "Agent with ID {} not found. Available agents in this app are: {}",
            agent_id,
            agent_names.join(", ")
        ).as_str())
    }

    fn discover_agents() -> Vec<bindings::exports::golem::agentic_guest::guest::Agent> {
        let agent_names = agent_registry::get_all_agent_definitions()
            .iter()
            .map(|x| x.agent_name.clone())
            .collect::<Vec<_>>();

        let mut agents = Vec::new();
        for agent_name in agent_names {
            let resolver = agent_registry::get_agent_impl_by_def(agent_name).unwrap();
            let agent = resolver.resolve_agent_impl();
            let agent = Agent::new(ResolvedAgent { agent });
            agents.push(agent);
        }

        agents
    }
}

impl GuestAgent for ResolvedAgent {
    fn get_agent_id(&self) -> String {
        self.agent.agent_id()
    }

    fn invoke(&self, method_name: String, input: Vec<String>) -> StatusUpdate {
        self.agent.invoke(method_name, input)
    }

    fn get_definition(&self) -> AgentDefinition {
        self.agent.get_definition()
    }
}

bindings::export!(Component with_types_in bindings);
