use event_store::*;
use futures::future::Future;
use log::trace;
use tokio::runtime::Runtime;

#[test]
fn cache_set_get() {
    pretty_env_logger::init();

    let test_entity = TestCounterEntity { counter: 100 };

    trace!("Save and emit test");

    let conn = pg_create_random_db();

    let rt = Runtime::new().unwrap();

    let run = pg_cache_save(conn.get().unwrap(), "_test".into(), &test_entity)
        .and_then(move |_| pg_cache_read::<TestCounterEntity>(conn.get().unwrap(), "_test".into()));

    let res = rt.block_on_all(run).unwrap();

    assert_eq!(res.1.unwrap().0, test_entity);
}
