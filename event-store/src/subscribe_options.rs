/// Subscribe options
#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    /// Whether to emit an event replay request when a subscription is started
    pub replay_previous_events: bool,

    /// Whether to save the event when it is received, or just pass it to the handler
    pub save_on_receive: bool,
}

// TODO: Is this struct still required?
impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            replay_previous_events: true,
            save_on_receive: true,
        }
    }
}
