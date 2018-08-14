use std::fmt::Debug;

// --- Event store crate ---
trait Event {}
trait Events {}
trait Aggregate<E: Events>: Copy + Clone + Debug + Default {
    fn apply_event(self, event: &E) -> Self;
}

// --- Example implementation for the Toaster domain ---
struct Increment(pub i32);
struct Decrement(pub i32);

impl Event for Increment {}
impl Event for Decrement {}

enum ToasterEvents {
    Inc(Increment),
    Dec(Decrement),
}

impl Events for ToasterEvents {}

#[derive(Debug, Copy, Clone, PartialEq)]
struct SomeDomainEntity {
    counter: i32,
}

impl Default for SomeDomainEntity {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl Aggregate<ToasterEvents> for SomeDomainEntity {
    fn apply_event(self, event: &ToasterEvents) -> Self {
        let counter = match event {
            ToasterEvents::Inc(inc) => self.counter + inc.0,
            ToasterEvents::Dec(dec) => self.counter - dec.0,
        };

        Self { counter, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_aggregates_events() {
        let events = vec![
            ToasterEvents::Inc(Increment(1)),
            ToasterEvents::Inc(Increment(1)),
            ToasterEvents::Dec(Decrement(2)),
            ToasterEvents::Inc(Increment(2)),
            ToasterEvents::Dec(Decrement(3)),
            ToasterEvents::Dec(Decrement(3)),
        ];

        let result: SomeDomainEntity = events
            .iter()
            .fold(SomeDomainEntity::default(), |acc, event| {
                acc.apply_event(event)
            });

        assert_eq!(result, SomeDomainEntity { counter: -4 });
    }
}
