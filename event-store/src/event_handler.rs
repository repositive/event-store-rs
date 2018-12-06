use crate::event::Event;

pub trait EventHandler: Sized {
    fn handle_event(event: Event<Self>);
}
