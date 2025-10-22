use crate::{
    agent_registry::get_agent_instance,
    bindings::{
        exports::golem::{
            agent::guest::Guest,
            api1_1_7::{
                load_snapshot::Guest as SnapshotGuest, save_snapshot::Guest as SaveSnapshotGuest,
            },
        },
        golem::agent::common::{AgentError, AgentType, DataValue},
    },
};

use golem_wasm_ast::analysis::analysed_type::str;

pub use agent_construct::*;
use golem_wasm_rpc::golem_rpc_0_2_x::types::ValueAndType;
pub use type_mapping::*;

pub mod agent;
mod agent_construct;
pub mod agent_instance_registry;
pub mod agent_registry;
pub mod bindings;
mod type_mapping;

#[derive(Clone)]
pub struct ResolvedAgent {
    pub agent: ::std::sync::Arc<dyn agent::Agent + Send + Sync>,
    pub agent_id: String,
}

struct Component;

impl SnapshotGuest for Component {
    fn load(bytes: Vec<u8>) -> Result<(), String> {
        todo!()
    }
}

impl SaveSnapshotGuest for Component {
    fn save() -> Vec<u8> {
        todo!()
    }
}

impl Guest for Component {
    fn initialize(agent_type: String, input: DataValue) -> Result<(), AgentError> {
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
            agent.initiate(input);
            Ok(())
        } else {
            panic!(
                "No agent implementation found for agent definition: {}",
                agent_type.type_name
            );
        }
    }

    fn invoke(method_name: String, input: DataValue) -> Result<DataValue, AgentError> {
        let resolved_agent = get_agent_instance();

        if let Some(agent) = resolved_agent {
            Ok(agent.agent.invoke(method_name, input))
        } else {
            Err(AgentError::CustomError(
                golem_wasm_rpc::ValueAndType::new(
                    golem_wasm_rpc::Value::String("No agent instance found".to_string()),
                    str(),
                )
                .into(),
            ))
        }
    }

    fn get_definition() -> AgentType {
        let resolved_agent = get_agent_instance();

        if let Some(agent) = resolved_agent {
            agent.agent.get_definition()
        } else {
            panic!("No agent instance found");
        }
    }

    fn discover_agent_types() -> Result<Vec<AgentType>, AgentError> {
        Ok(agent_registry::get_all_agent_definitions())
    }
}

bindings::export!(Component with_types_in bindings);
