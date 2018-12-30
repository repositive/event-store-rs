mod cache;
mod emitter;
mod store;

pub use self::cache::{CacheResult, PgCacheAdapter};
pub use self::emitter::AmqpEmitterAdapter;
pub use self::store::{PgQuery, PgStoreAdapter};

#[derive(Debug, Clone)]
pub struct SubscribeOptions {
    pub(crate) replay_previous_events: bool,
    pub(crate) save_on_receive: bool,
}

impl Default for SubscribeOptions {
    fn default() -> Self {
        Self {
            replay_previous_events: true,
            save_on_receive: true,
        }
    }
}
