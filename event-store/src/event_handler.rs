use crate::event::Event;
use crate::store::Store;

pub trait EventHandler: Sized {
    fn handle_event(event: Event<Self>, saver: &Store);
}
