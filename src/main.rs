use futures::{StreamExt, TryStreamExt};
use futures_timer;
use heim::{
    process::{self, Process, ProcessError, ProcessResult},
    units::{
        ratio,
        Ratio,
    },
};
use warp::{Filter};
use std::time::Duration;
use heim::process::CpuUsage;
use std::cmp::Ordering::Equal;


#[tokio::main]
async fn main() {
    let hello_world = warp::path("test")
        .and_then(test);

    warp::serve(hello_world)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

async fn test() -> Result<String, warp::Rejection> {
    // let mut proc_stream = heim::process::processes().await.unwrap()
    //     .filter_map(|proc_res| async move {
    //         if let Ok(p) = proc_res { Some(p) } else { None }
    //     })
    //     .filter_map(|proc| async move {
    //         if let Ok(name) = proc.name().await { Some(name) } else { None }
    //     })
    //     .collect::<Vec<_>>()
    //     .await;
    // futures::pin_mut!(proc_stream);


    let mut processes = heim::process::processes().await.unwrap()
        .map_ok(|proc| usage(proc))
        .try_buffer_unordered(usize::MAX)
        .filter_map(|res| async move {
            if let Ok(usage) = res { Some(usage) } else { None }
        })
        .collect::<Vec<_>>()
        .await;

    processes.sort_by(|a: &(String, f32), b: &(String, f32)| {
        b.1.partial_cmp(&a.1).unwrap_or(Equal)
    });


    Ok(format!("{:#?}", processes))
}

async fn usage(process: Process) -> ProcessResult<(String, f32)> {
    let name = process.name().await?;
    let usage_1 = process.cpu_usage().await?;
    futures_timer::Delay::new(Duration::from_millis(100)).await;
    let usage_2 = process.cpu_usage().await?;
    let delta = (usage_2 - usage_1).get::<ratio::ratio>() * 100.0;

    Ok((name, delta))
}
