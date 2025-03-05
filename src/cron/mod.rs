mod cron_price;
mod cron_rate;

use tokio_cron_scheduler::{Job, JobScheduler};

use crate::core::Store;
use crate::util::LibResult;

pub async fn run(store: Store) -> LibResult<()> {
    // start cron job
    let sched = JobScheduler::new().await?;
    let store1 = store.clone();

    let price_job = Job::new_async("5 */10 * * * *", move |_, _| {
        let stores = store1.clone();
        Box::pin(async move {
            let cron = cron_price::CronPrice::new(stores);
            cron.run().await.expect("TODO: panic message");
        })
    })?;

    let rate_job = Job::new_async("5 0 * * * *", move |_, _| {
        let stores = store.clone();
        Box::pin(async move {
            let cron = cron_rate::CronRate::new(stores);
            cron.run().await.expect("TODO: panic message");
        })
    })?;

    sched.add(price_job).await?;
    sched.add(rate_job).await?;
    sched.start().await?;
    Ok(())
}
