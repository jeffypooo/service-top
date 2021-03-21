#[macro_use] extern crate log;
extern crate pretty_env_logger;

use std::cmp::Ordering::Equal;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::{StreamExt, TryStreamExt};
use futures::executor::block_on;
use futures_timer;
use heim::{
    process::{self, Process, ProcessError, ProcessResult},
    units::{
        ratio,
        Ratio,
    },
};
use heim::process::CpuUsage;
use tokio::sync::broadcast::Receiver;
use warp::Filter;

use crate::procs::ProcessesHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init_timed();

    info!("Service Top v0.1.0");

    let process_handler = ProcessesHandler::new();
    let server_handler = process_handler.clone();
    let updates_handler = process_handler.clone();
    let server = tokio::spawn(async move {
        let procs = procs::api_filters(server_handler)
            .with(warp::log("api::procs"));
        warp::serve(procs)
            .run(([127, 0, 0, 1], 8080))
            .await;
    });

    let process_updates = tokio::spawn(async move {
        match updates_handler.processes_loop().await {
            Ok(_) => println!("process loop exit?"),
            Err(e) => panic!("process loop error: {}", e)
        }
    });

    tokio::join!(server, process_updates);
    Ok(())
}

mod procs {
    use std::cmp::Ordering::Equal;
    use std::convert::Infallible;
    use std::error::Error;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use futures::{StreamExt, TryStreamExt};
    use futures_timer::Delay;
    use heim::process;
    use heim::process::{Process, ProcessResult};
    use heim::units::ratio;
    use tokio::task::JoinHandle;
    use warp::Filter;

    #[derive(Clone)]
    pub struct ProcessesHandler {
        process_list: Arc<Mutex<Vec<(String, f32)>>>
    }

    impl ProcessesHandler {
        pub fn new() -> ProcessesHandler {
            let list = Vec::new();
            ProcessesHandler { process_list: Arc::new(Mutex::new(list)) }
        }
    }

    impl ProcessesHandler {

        pub async fn processes_loop(&self) -> Result<(), Box<dyn std::error::Error>> {
            let tick_rate = Duration::from_millis(500);
            loop {
                let beg = tokio::time::Instant::now();
                let mut usages = process::processes().await.unwrap()
                    .map_ok(|p| usage(p))
                    .try_buffer_unordered(std::usize::MAX)
                    .filter_map(|res| async move {
                        if let Ok(usage) = res { Some(usage) } else { None }
                    })
                    .filter(|usg| futures::future::ready(usg.1 > 0.0))
                    .collect::<Vec<_>>()
                    .await;
                usages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Equal));
                self.update_usages(usages);

                let sleep_dur = tick_rate.checked_sub(beg.elapsed())
                    .unwrap_or(Duration::from_secs(0));

                // No need to refresh as fast as possible
                tokio::time::sleep(sleep_dur).await;
            }

            Ok(())
        }

        fn update_usages(&self, new_list: Vec<(String, f32)>) {
            let mut cur_list = self.process_list.lock().unwrap();
            *cur_list = new_list;
            // debug!("procs list updated");
        }

        pub async fn list_procs(&self) -> Result<impl warp::Reply, Infallible> {
            let value = self.process_list.lock().unwrap();
            let resp = format!("{:#?}", value);
            Ok(resp)
        }
    }

    async fn usage(process: Process) -> ProcessResult<(String, f32)> {
        let name = process.name().await?;
        let usage_1 = process.cpu_usage().await?;
        futures_timer::Delay::new(Duration::from_millis(250)).await;
        let usage_2 = process.cpu_usage().await?;
        let delta = (usage_2 - usage_1).get::<ratio::ratio>() * 100.0;

        Ok((name, delta))
    }

    pub fn api_filters(handler: ProcessesHandler) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
        warp::path("test")
            .and(warp::get())
            .and(with_handler(handler))
            .and_then(|p: ProcessesHandler| async move {
                p.list_procs().await
            })
    }

    fn with_handler(handler: ProcessesHandler) -> impl Filter<Extract=(ProcessesHandler, ), Error=std::convert::Infallible> + Clone {
        warp::any().map(move || handler.clone())
    }
}

