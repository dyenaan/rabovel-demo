use std::sync::Arc;
pub(crate) trait AccountStore {
    fn accounts(&self) -> &Arc<dyn identity::accounts::ports::AccountRepository>;
}
pub(crate) trait AccountLookup {
    fn account_queries(&self) -> &Arc<dyn identity::accounts::ports::AccountQueries>;
}
pub(crate) trait ProfileUpdates {
    fn profile_updates(&self) -> &Arc<dyn identity::accounts::ports::ProfileUpdates>;
}
