use warp::Filter;

#[tokio::main]
async fn main() {
    println!("VoiceBBS Server is starting...");

    // Basic route to confirm server is running
    let hello = warp::path::end()
        .map(|| warp::reply::html("VoiceBBS is alive!"));

    warp::serve(hello)
        .run(([0, 0, 0, 0], 8080))
        .await;
}