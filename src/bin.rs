use std::thread;

extern crate podium_lib;
use podium_lib::contracts::AppState::*;
use podium_lib::query_executor::QueryResponse;
use podium_lib::routes::search;
use podium_lib::tantivy_process::start_tantivy;

#[macro_use]
extern crate log;

use std::io;

use actix_cors::Cors;
use actix_web::{http, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use crossbeam::channel::{unbounded, Receiver, Sender};

#[tokio::main]
async fn main() -> io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let local = tokio::task::LocalSet::new();

    // Set up communication channels
    let (query_tx, query_rx): (Sender<String>, Receiver<String>) = unbounded();
    let (result_tx, result_rx): (Sender<QueryResponse>, Receiver<QueryResponse>) = unbounded();
    let tantivy_query_tx = query_tx.clone();

    let tantivy_thread = tokio::spawn(async move {
        start_tantivy((tantivy_query_tx, query_rx), result_tx).await;
    });

    let sys = actix_rt::System::run_in_tokio("server", &local);

    let app_state = web::Data::new(AppState {
        query_sender: query_tx.clone(),
        result_receiver: result_rx.clone(),
    });

    let server_res = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::new() // <- Construct CORS middleware builder
                    .send_wildcard()
                    .finish(),
            )
            .app_data(app_state.clone())
            .configure(search::config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await?;

    sys.await?;

    Ok(server_res)

    // if tantivy_thread.unwrap().join().is_err() {
    //     error!("Failed to join tantivy thread");
    // }
}
