use crate::binding;
use crate::binding::exports::golem::agentic::guest::{AgentDefinition, Guest, GuestAgent, StatusUpdate};

wit_bindgen::generate!({
    path: "wit",
    world: "agentic",
    generate_all,
    generate_unused_types: true,
    additional_derives: [PartialEq, golem_rust::FromValueAndType, golem_rust::IntoValue],
    pub_export_macro: true,
});
