use crate::bindings::exports::golem::agentic_guest::guest::StatusUpdate;
use crate::bindings::exports::golem::agentic_guest::guest::{AgentDefinition, Guest, GuestAgent};

pub mod agent;
pub mod agent_registry;
pub mod bindings;
pub mod agent_instance_counter;

pub struct ResolvedAgent {
    pub agent: ::std::sync::Arc<dyn agent::Agent + Send + Sync>,
}

struct Component;

impl Guest for Component {
    type Agent = ResolvedAgent;

    fn discover_agent_definitions() -> Vec<AgentDefinition> {
        agent_registry::get_all_agent_definitions()
    }
}

impl GuestAgent for ResolvedAgent {
    fn new(agent_name: String) -> Self {
        let agent_definitions = agent_registry::get_all_agent_definitions();

        let agent_definition = agent_definitions.iter().find(|x| x.agent_name == agent_name).expect(
            format!("Agent definition not found for agent name: {}. Available agents in this app is {}", agent_name,
                agent_definitions.iter().map(|x| x.agent_name.clone()).collect::<Vec<_>>().join(", ")).as_str()
        );

        let agent_impl_resolver =
            agent_registry::get_agent_impl_by_def(agent_definition.agent_name.clone());

        if let Some(resolver) = agent_impl_resolver {
            let agent = resolver.resolve_agent_impl(agent_name);
            ResolvedAgent { agent }
        } else {
            panic!(
                "No agent implementation found for agent definition: {}",
                agent_name
            );
        }
    }

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
