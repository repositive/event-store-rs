- Move event metadata creation out of store adapter
- How do we handle events from other domains?
	- Custom attr on enum
	- What about enums of events from other domains?
		- One enum for every event
	- How does this get passed into the aggregator?


Rename `Events` to `EventData`:

```
struct Event<C: EventContext, D: EventData> {
    id: Uuid,
    context: Option<C>,
    data: D,
}

impl Event<C, D> where ... {
	pub fn new(data: D, context: C, id: Uuid) -> Self {

	}
}
```

- Change crate name to `event-store`
- `#[event_store(namespace = "some_ns")]`
- `Event` struct should have methods to get namespace and event type

---

How Serde parses #[serde(tag = "thing")]: https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs#L307-L319
	It's on `Container` because it's a container tag
Deciding a tag (internal/external?): https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs#L462
Deciding an identifier: https://github.com/serde-rs/serde/blob/master/serde_derive/src/internals/attr.rs#L515

---

TODO

- AMQP in event store
- Webserver (actix-web)
- Sendgrid driver using either gsquire/sendgrid-rs or a library in a workspace
- R2D2 support in event store postgres