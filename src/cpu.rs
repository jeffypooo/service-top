pub mod processor {
    pub mod api {

        use warp::{Filter, Rejection, Reply};

        pub fn routes() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
            let load = warp::path("load")
                .and(warp::get())
                .and(warp::path::end())
                .map(|| "TODO");
            let sensors = warp::path("sensors")
                .and(warp::get())
                .and(warp::path::end())
                .map(|| "TODO");
            warp::path("cpu").and(load.or(sensors))
        }
    }
}
