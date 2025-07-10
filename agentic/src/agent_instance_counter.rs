use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Mutex;
use once_cell::sync::Lazy;

pub type AgentName = String;
pub type AgentId = String;

static AGENT_INSTANCE_COUNTER: Lazy<Mutex<HashMap<AgentName, u64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static AGENT_INSTANCE_ID: Lazy<Mutex<HashMap<AgentName, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub fn increment_agent_instance_counter(agent_name: AgentName) -> u64 {
    let mut counter = AGENT_INSTANCE_COUNTER.lock().unwrap();
    let count = counter.entry(agent_name).or_insert(0);
    *count += 1;
    *count
}

pub fn create_agent_id(agent_name: AgentName)  {
    let mut id_map = AGENT_INSTANCE_ID.lock().unwrap();

    let count = increment_agent_instance_counter(agent_name.clone());

    let worker_name =
        crate::bindings::golem::api::host::get_self_metadata().worker_id.worker_name.clone();

    let agent_id = format!("{}-{}-{}", worker_name, agent_name, count);

    id_map.insert(agent_name.clone(), agent_id.clone());
}

pub fn get_agent_id(agent_name: AgentName) -> String {
    let id_map = AGENT_INSTANCE_ID.lock().unwrap();
    id_map.get(&agent_name).cloned().unwrap()
}

pub fn get_agent_instance_count(agent_name: AgentName) -> u64 {
    let counter = AGENT_INSTANCE_COUNTER.lock().unwrap();
    *counter.get(&agent_name).unwrap_or(&0)
}
