#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    pub replay_previous_events: bool,
    pub save_on_receive: bool,
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            replay_previous_events: true,
            save_on_receive: true,
        }
    }
}
