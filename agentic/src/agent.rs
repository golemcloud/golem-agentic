use crate::binding::exports::golem::agentic::guest::StatusUpdate;

// A simple Agent that every agent abstraction has to extend
pub trait Agent: Send + Sync {
    fn raw_agent_id(&self) -> String;
    fn invoke(&self, method_name: String, input: Vec<String>) -> StatusUpdate;
}