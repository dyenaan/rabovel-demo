use std::sync::Arc;
pub(crate) trait EventPublisher {
    fn event_producer(&self) -> &Arc<dyn events::EventProducer>;
}
