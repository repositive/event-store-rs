mod event;
mod event_store;

pub use crate::event_store::EventStore;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
