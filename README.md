              _____                _     _____ _                 
             |  ___|              | |   /  ___| |                
             | |____   _____ _ __ | |_  \ `--.| |_ ___  _ __ ___ 
             |  __\ \ / / _ \ '_ \| __|  `--. \ __/ _ \| '__/ _ \
             | |___\ V /  __/ | | | |_  /\__/ / || (_) | | |  __/
             \____/ \_/ \___|_| |_|\__| \____/ \__\___/|_|  \___|

[![Build Status](https://travis-ci.org/repositive/event-store-rs.svg?branch=master)](https://travis-ci.org/repositive/event-store-rs)

# Index


- What is Event Store?
- Motivation
- API Documentation
- Architecture
- Implemented Backends

# What is Event Store?

Is an open source implementation of an event store core logic and [anticorruption layer](https://docs.microsoft.com/en-us/azure/architecture/patterns/anti-corruption-layer).
An event store is a software design patten to deal with state by keeping a log of events.

It's premise; never delete or mutate the stored events and derive the state of the system from them. 

# API Documentation

```bash
cargo doc --open
```


# Motivation


* In systems with multiple services, there's lots of moving parts
  - Promotes high cohesion with low coupling.
  - Easier to synchronize

* Dealing with a shapeless ever-evolving schema
  - Change the view without modifying the underlaying structure.
  - Reduced breaking changes - Easier to maintain
  - Increased agility

* Enabling Reproducibility
  - Easier to find, identify and resolve bugs.
  - High quality audit logs in case of a security breach.

* Improving Analytics
  - Data collection is a default.
  - More oportunities for data driven approaches.

# API Docs

```bash
cargo doc --open
```

# Architecture

                 +------------------------+
                 |                        |       +------------------------+
                 |          CORE          |       |                        |
                 |                        |       |        ADAPTERS        |
                 |  +------------------+  |       |                        |
                 |  |                  |  |       |  +------------------+  |
                 |  |    INTERFACE     |  |       |  |                  |  |
                 |  |                  |  |    ------>       STORE      |  |
                 |  +--------+----------  |    |  |  |                  |  |
                 |           |            |    |  |  +------------------+  |
                 |           |            |    |  |                        |
                 |  +--------v----------  |    |  |  +------------------+  |
                 |  |                  |  |    |  |  |                  |  |
                 |  |     BACKEND      +------------->       CACHE      |  |
                 |  |                  |  |    |  |  |                  |  |
                 |  +------------------+  |    |  |  +------------------+  |
                 |                        |    |  |                        |
                 |                        |    |  |  +------------------+  |
                 +------------------------+    |  |  |                  |  |
                                               +----->      EMITTER     |  |
                                                  |  |                  |  |
                                                  |  +------------------+  |
                                                  |                        |
                                                  |                        |
                                                  +------------------------+

Implemented Backends
---
Store Adapter:
  - Postgres
Cache Adapter:
  - Postgres
Emitter Adadpter:
  - AMQP
