use lapin_futures::client::Client as LapinClient;
use postgres::Client as PgClient;
use r2d2_postgres::PostgresConnectionManager;
use tokio::net::TcpStream;

pub struct EventStore {
    pool: PostgresConnectionManager<PgClient>,
    queue: LapinClient<TcpStream>,
}

impl EventStore {
    pub fn new(pool: PostgresConnectionManager<PgClient>, queue: LapinClient<TcpStream>) -> Self {
        Self { pool, queue }
    }
}
