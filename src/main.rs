mod evt;
mod entity;
mod util;
mod core;
mod svc;
mod cron;

#[tokio::main]
async fn main() {
    util::log::init();
    // init log 
    let store = core::pool::init_pool().await;

    // start cron time
    let cron_store = store.clone();
    tokio::spawn(async move {
        if let Err(e) = cron::run(cron_store).await {
            tracing::error!("cron err={e}")
        }
    });

    // start evt monitor
    let evt_monitor = evt::Evt::new(store);
    if let Err(e) = evt_monitor.run().await {
        tracing::error!("evt monitor err={e}")
    }
}
