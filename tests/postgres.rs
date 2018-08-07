extern crate event_store_rs;
extern crate r2d2;
extern crate r2d2_postgres;

#[feature(postgres_support)]
mod postgres {
    use event_store_rs::*;
    use r2d2;
    use r2d2_postgres::{PostgresConnectionManager, TlsMode};

    #[test]
    // Ensure that `PostgresEventStore` exists with the Postgres feature enabled
    fn there_is_a_postgres_backed_store() {
        let manager =
            PostgresConnectionManager::new("postgres://postgres@localhost:5430", TlsMode::None)
                .expect("Could not open connection to database");
        let pool = r2d2::Pool::new(manager).expect("Could not create pool");

        PostgresEventStore::new(pool);
    }
}
