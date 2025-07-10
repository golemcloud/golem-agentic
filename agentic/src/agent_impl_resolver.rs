use std::sync::Arc;

pub trait Resolver: Send + Sync {
    fn resolve_agent_impl(
        &self,
        agent_name: String,
    ) -> Arc<dyn Agent + Send + Sync>;
}
