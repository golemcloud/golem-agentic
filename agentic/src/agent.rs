use crate::bindings::exports::golem::agentic_guest::guest::{AgentDefinition, StatusUpdate};

// A simple Agent that every agent abstraction has to extend
// This is auto implemented when using `agent_implementation` attribute.
// Implementation detail: Once the agent_impl trait has an instance of `Agent`,
// it's internal functionalities can be used to further implement the real component
//
// We never want to directly implement this trait
// Example usage:
//
// ```
//  [agent_definition]
//  trait WeatherAgent: Agent {
//    fn get_weather(&self, location: String) -> String;
//  }
// ```
//
//  ```
//  struct MyWeatherAgent;
//
//  #[agent_implementation]
//  impl WeatherAgent for MyWeatherAgent {fn get_weather(&self, location: String) -> String } }
//  ```
// There is no need to implement `Agent` anywhere, as it is automatically implemented by the `[agent_implementation]` attribute.
pub trait Agent: Send + Sync {
    fn agent_id(&self) -> String;
    fn invoke(&self, method_name: String, input: Vec<String>) -> StatusUpdate;
    fn get_definition(&self) -> AgentDefinition;
}

pub trait GetAgentId {
    fn get_agent_id() -> String;
}
