mod evt;
mod entity;
mod util;
mod core;
mod svc;

#[tokio::main]
async fn main() {
    util::log::init();
    // 初始化数据库链接
    let store = core::pool::init_pool().await;

    // 启动定时任务
    // let cron_store = store.clone();

    // 启动eth事件监听
    let evt_monitor = evt::Evt::new(store);
    if let Err(e) = evt_monitor.run().await {
        tracing::error!("evt monitor err={e}")
    }
}
