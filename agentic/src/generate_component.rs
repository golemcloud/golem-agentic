// TODO;May be it doesn't need to be macros, as we never need to embed this anywhere else than SDK

#[macro_export]
macro_rules! generate_component {
    () => {
        pub struct ResolvedAgent {
            pub agent: ::std::sync::Arc<dyn crate::agent::Agent + Send + Sync>,
        }

        struct Component;

        impl crate::bindings::exports::golem::agentic::guest::Guest for Component {
            type Agent = ResolvedAgent;

            fn discover_agent_definitions() -> Vec<crate::bindings::exports::golem::agentic::guest::AgentDefinition> {
                crate::agent_registry::get_all_agent_definitions()
            }
        }

        impl crate::bindings::exports::golem::agentic::guest::GuestAgent for ResolvedAgent {
            fn new(agent_name: String, agent_id: String) -> Self {
                let agent_definitions = crate::agent_registry::get_all_agent_definitions();
                let agent_definition = agent_definitions.iter().find(|x| x.agent_name == agent_name).unwrap();
                let agent_impl_resolver = crate::agent_registry::get_agent_impl_by_def(agent_definition.agent_name.clone());
                if let Some(resolver) = agent_impl_resolver {
                    let agent = resolver.resolve_agent_impl(agent_id, agent_name);
                    ResolvedAgent { agent }
                } else {
                    panic!("No agent implementation found for agent definition: {}", agent_name);
                }
            }

            fn invoke(&self, method_name: String, input: Vec<String>) -> crate::bindings::exports::golem::agentic::guest::StatusUpdate {
                self.agent.invoke(method_name, input)
            }

            fn get_definition(&self) -> crate::bindings::exports::golem::agentic::guest::AgentDefinition {
                self.agent.get_definition()
            }
        }

        crate::bindings::export!(Component with_types_in crate::bindings);
    };
}
