pub mod memory {
    use heim;
    use serde::Serialize;
    use serde_json;

    #[derive(Serialize, Debug)]
    pub struct MemoryInfo {
        total: u64,
        used: u64,
        available: u64,
    }

    pub async fn get_memory_info() -> Result<MemoryInfo, Box<dyn std::error::Error>> {
        let mem = heim::memory::memory().await?;
        let info = MemoryInfo {
            total: mem.total().value,
            used: (mem.available() - mem.free()).value,
            available: mem.available().value,
        };
        Ok((info))
    }

    pub mod api {
        use std::convert::Infallible;

        use tokio::sync::watch;
        use warp::Filter;

        use super::MemoryInfo;

        pub fn routes(
            rx: watch::Receiver<MemoryInfo>,
        ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
            let total = warp::path("total")
                .and(warp::get())
                .and(warp::path::end())
                .and(warp::any().map(move || rx.clone()))
                .and_then(|mut rx: watch::Receiver<MemoryInfo>| async move {
                    match rx.changed().await {
                        Ok(_) => {
                            let info = &*rx.borrow();
                            Ok(warp::reply::json(&info))
                        }
                        Err(e) => Err(warp::reject::not_found()),
                    }
                });

            warp::path("memory").and(total)
        }

        async fn total_json() -> Result<impl warp::Reply, Infallible> {
            let info = super::get_memory_info().await.unwrap();
            Ok(warp::reply::json(&info))
        }
    }
}
