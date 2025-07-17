use crate::bindings::exports::golem::agent::guest::{AgentType, StatusUpdate};
use golem_wasm_rpc::WitValue;
use crate::AgentConstruct;

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
    fn get_id(&self) -> String;
    fn invoke(&self, method_name: String, input: Vec<WitValue>) -> StatusUpdate;
    fn get_definition(&self) -> AgentType;
}

pub trait GetAgentId {
    fn get_agent_id() -> String;
}

pub struct AgentInfo {
    pub agent_name: String,
    pub worker_name: String,
    pub instance_number: String,
}

pub fn parse_agent_id(agent_id: &str) -> Result<AgentInfo, String> {
    let parts: Vec<&str> = agent_id.split("--").collect();
    if parts.len() < 3 {
        return Err(format!(
            "Invalid agent_id format: {}. Expected format is {{worker_name}}-{{agent_name}}-{{instance_number}}",
            agent_id
        ));
    }

    let worker_name = parts[0].to_string();
    let agent_name = parts[1].to_string();
    let instance = parts[2].to_string();

    Ok(AgentInfo {
        agent_name,
        worker_name,
        instance_number: instance,
    })
}
