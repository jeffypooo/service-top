use warp::Filter;
use std::io::Error;
use prettytable::{
    Table,
    Row,
    Cell
};

use futures::{
    executor,
    stream,
    Stream,
    StreamExt
};
use heim;

#[tokio::main]
async fn main() {
    let hello_world = warp::path("test")
        .and_then(test);

    warp::serve(hello_world)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

async fn test() -> Result<String, warp::Rejection> {

    let f = heim::process::processes()
        .await
        .unwrap()
        .into_future();



    Ok(format!("{:#?}", ""))
}
