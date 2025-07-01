wit_bindgen::generate!({
    path: "wit",
    world: "agentic-guest",
    generate_all,
    generate_unused_types: true,
    additional_derives: [PartialEq],
    pub_export_macro: true,
});
