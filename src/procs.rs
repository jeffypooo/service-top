pub mod processes {
    use std::cmp::Ordering::Equal;
    use std::convert::Infallible;
    use std::ops::Deref;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use futures::{future, StreamExt, TryFutureExt, TryStreamExt};
    use heim::process;
    use heim::process::{Process, ProcessResult};
    use heim::units::ratio;
    use serde::ser::SerializeStruct;
    use serde::{Serialize, Serializer};

    // #[derive(Serialize, Deserialize)]
    // #[serde(rename_all = "camelCase")]
    #[derive(Debug)]
    pub struct ProcessInfo {
        pub name: String,
        pub cpu_usage: f32,
    }

    impl Serialize for ProcessInfo {
        fn serialize<S>(
            &self,
            serializer: S,
        ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
        where
            S: Serializer,
        {
            let cpu_pct = format!("{:.2}%", &self.cpu_usage * 100.0);
            let mut state = serializer.serialize_struct("ProcessInfo", 3).unwrap();
            state.serialize_field("name", &self.name)?;
            state.serialize_field("cpu", &cpu_pct)?;
            state.end()
        }
    }

    pub async fn list_usages() -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error>> {
        let mut usages = heim::process::processes()
            .await
            .unwrap()
            .map_ok(|proc| usage(proc))
            .try_buffer_unordered(usize::MAX)
            .filter_map(|res| async move {
                if let Ok(info) = res {
                    Some(info)
                } else {
                    None
                }
            })
            .filter(|info| future::ready(info.cpu_usage > 0.0))
            .collect::<Vec<ProcessInfo>>()
            .await;

        usages.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(Equal));

        Ok(usages)
    }

    pub async fn usage(process: Process) -> ProcessResult<ProcessInfo> {
        let name = process.name().await?;
        let usage_1 = process.cpu_usage().await?;
        futures_timer::Delay::new(Duration::from_millis(500)).await;
        let usage_2 = process.cpu_usage().await?;
        let delta = (usage_2 - usage_1).get::<ratio::ratio>() * 100.0;

        Ok(ProcessInfo {
            name,
            cpu_usage: delta,
        })
    }

    pub mod api {
        use crate::procs::processes::ProcessInfo;
        use tokio::sync::watch;
        use warp::Filter;

        pub fn routes(
            mut rx: watch::Receiver<Vec<ProcessInfo>>,
        ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
            let list = warp::path::end()
                .and(warp::get())
                .and(warp::any().map(move || rx.clone()))
                .and_then(|mut rx: watch::Receiver<Vec<ProcessInfo>>| async move {
                    match rx.changed().await {
                        Ok(_) => {
                            let info = &*rx.borrow();
                            Ok(warp::reply::json(&info))
                        }
                        Err(e) => Err(warp::reject::not_found()),
                    }
                });
            warp::path("processes").and(list)
        }
    }
}
