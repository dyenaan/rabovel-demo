/// Failure reported by a purchase provider. A potentially applied chain effect
/// requires reconciliation before retrying; this does not persist recovery state.
#[derive(Debug)]
pub struct PurchaseExecutionError {
    pub message: String,
    pub reconciliation_required: bool,
}
