use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

pub type AgentName = String;
pub type AgentId = String;

static AGENT_INSTANCE_COUNTER: Lazy<Mutex<HashMap<AgentName, u64>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static AGENT_INSTANCE_ID: Lazy<Mutex<HashMap<AgentName, Vec<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn increment_agent_instance_counter_locked(
    counter: &mut HashMap<AgentName, u64>,
    agent_name: &AgentName,
) -> u64 {
    let count = counter.entry(agent_name.clone()).or_insert(0);
    *count += 1;
    *count
}

pub fn increment_agent_instance_counter(agent_name: AgentName) -> u64 {
    let mut counter = AGENT_INSTANCE_COUNTER.lock().unwrap();
    increment_agent_instance_counter_locked(&mut counter, &agent_name)
}

pub fn create_agent_id(agent_name: AgentName) -> AgentId {
    let mut counter = AGENT_INSTANCE_COUNTER.lock().unwrap();
    let count = increment_agent_instance_counter_locked(&mut counter, &agent_name);

    let worker_name = crate::bindings::golem::api::host::get_self_metadata()
        .worker_id
        .worker_name
        .clone();

    let agent_id = format!("{}--{}--{}", worker_name, agent_name, count);

    // Lock second to maintain order
    let mut id_map = AGENT_INSTANCE_ID.lock().unwrap();
    id_map.entry(agent_name).or_default().push(agent_id.clone());

    agent_id
}

pub fn get_agent_instance_count(agent_name: AgentName) -> u64 {
    let counter = AGENT_INSTANCE_COUNTER.lock().unwrap();
    *counter.get(&agent_name).unwrap_or(&0)
}
