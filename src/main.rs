#[macro_use]
extern crate log;
extern crate pretty_env_logger;

use std::borrow::Borrow;
use std::time::Duration;

use futures::TryFutureExt;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use tokio::sync::watch;
use tokio::time::Instant;
use warp::Filter;

use cpu::processor;
use mem::memory;
use mem::memory::MemoryInfo;
use procs::processes;

mod cpu;
mod mem;
mod procs;

static DUR_ZERO: Duration = Duration::from_secs(0);
static TICK_RATE: Duration = Duration::from_millis(500);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init_timed();

    info!("Service Top v0.1.0");

    let (proc_tx, mut proc_rx) = watch::channel(processes::list_usages().await?);
    let (mem_tx, mut mem_rx) = watch::channel(memory::get_memory_info().await?);

    let memory_updates = tokio::spawn(async move {
        let last_tick = Instant::now();
        loop {
            let info = memory::get_memory_info().await.unwrap();
            mem_tx.send(info).unwrap_or_else(|e| error!("{}", e));

            let sleep_dur = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or(DUR_ZERO);
            tokio::time::sleep(sleep_dur).await;
        }
    });

    let process_updates = tokio::spawn(async move {
        let last_tick = Instant::now();
        loop {
            let usages = processes::list_usages().await.unwrap();
            proc_tx.send(usages).unwrap_or_else(|e| error!("{}", e));

            let sleep_dur = TICK_RATE
                .checked_sub(last_tick.elapsed())
                .unwrap_or(DUR_ZERO);
            tokio::time::sleep(sleep_dur).await;
        }
    });

    let server = tokio::spawn(async move {
        let top = warp::path("top")
            .and(
                processes::api::routes(proc_rx)
                    .or(memory::api::routes(mem_rx))
                    .or(processor::api::routes()),
            )
            .with(warp::cors::cors().allow_any_origin())
            .with(warp::log("top::api"));

        warp::serve(top).run(([127, 0, 0, 1], 8080)).await;
    });

    let (_, _, _) = tokio::join!(server, process_updates, memory_updates);

    Ok(())
}
