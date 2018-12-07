use crate::event::Event;
use crate::event_saver::EventSaver;

pub trait EventHandler: Sized {
    fn handle_event(event: Event<Self>, saver: EventSaver);
}
