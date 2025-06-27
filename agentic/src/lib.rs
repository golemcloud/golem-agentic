use crate::binding::export;
use crate::binding::exports::golem::agentic::guest::{
    AgentDefinition, Guest, GuestAgent, StatusUpdate,
};

pub mod agent;
pub mod agent_registry;
pub mod binding;

// Under the hood (Remove this code once done)
//
// struct Component;
//
// impl Guest for Component {
//     type Agent = Agents;
//
//     fn discover_agent_definitions() -> Vec<AgentDefinition> {
//         vec![]
//     }
// }
//
// trait Agent {
//     fn invoke_methods(&self, method_name: String, input: Vec<String>) -> StatusUpdate;
//
// }
//
// impl Agent for MyAgent1 {
//     fn invoke_methods(&self, method_name: String, input: Vec<String>) -> StatusUpdate {
//         if method_name == "greet" {
//             StatusUpdate::Emit(format!("Hello, {}", input.join(", ")))
//         } else {
//             StatusUpdate::Emit("Unknown method".to_string())
//         }
//     }
// }
//
// impl Agent for MyAgent2 {
//     fn invoke_methods(&self, method_name: String, input: Vec<String>) -> StatusUpdate {
//         if method_name == "farewell" {
//             StatusUpdate::Emit(format!("Goodbye, {}", input.join(", ")))
//         } else {
//             StatusUpdate::Emit(format!("Unknown method: {}", method_name))
//         }
//     }
// }
//
//
// struct Agents {
//     agent: Box<dyn Agent>
// }
//
// struct MyAgent1 {
//     agent_id: String
// }
//
// struct MyAgent2 {
//     agent_id: String
// }
//
// impl GuestAgent for Agents {
//     fn new(agent_name: String, agent_id: String) -> Self {
//         if agent_name == "afsal" {
//             // If agent-name is present then it implies we can check the locally registered
//             // agent definitions, or else discover the agent definitions
//             Agents {
//                 agent: Box::new(MyAgent1 {agent_id}),
//             }
//         } else if agent_name == "golem" {
//             Agents {
//                 agent: Box::new(MyAgent2 {agent_id}),
//             }
//         } else {
//             panic!("Unknown agent name: {}", agent_name);
//         }
//     }
//
//     fn invoke(&self, method_name: String, input: Vec<String>) -> StatusUpdate {
//         self.agent.invoke_methods(method_name, input)
//     }
//
//     fn get_definition(&self) -> AgentDefinition {
//         AgentDefinition {
//             agent_name: "".to_string(),
//             description: "".to_string(),
//             methods: vec![],
//             requires: vec![],
//         }
//     }
// }
//
// export!(Component with_types_in binding);
