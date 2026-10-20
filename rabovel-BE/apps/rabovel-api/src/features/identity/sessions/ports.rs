use std::sync::Arc;
pub(crate) trait SessionStore {
    fn sessions(&self) -> &Arc<dyn identity::sessions::ports::SessionRepository>;
}
