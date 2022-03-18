use actix_web::{middleware, web, App, HttpServer};
use env_logger;
use http_server::*;
use std::{env, sync::Arc};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let args = env::args();
    let mut dir: Option<String> = None;
    let mut _options: Vec<Flags> = Vec::new();

    // parse arguments
    for arg in args.skip(1) {
        if let Some('-') = arg.chars().next() {
            println!("Adding flag: {}", arg);
            match arg.as_str() {
                // todo: fill out options as they are needed (none yet)
                _ => panic!("unknown flag '{}'", arg),
            }
        } else {
            if dir == None {
                dir = Some(arg);
            } else {
                panic!("invalid command format - unexpected argument: '{}'", arg);
            }
        }
    }

    if dir == None {
        println!("Usage:\n\tcargo run -- [DIRECTORY]\n");
    }

    let dir = dir.expect("no directory provided");
    let content = Arc::new(load_website(&dir).expect("unable to load from directory"));

    HttpServer::new(move || {
        let content = content.clone();

        App::new()
            .wrap(middleware::Logger::default())
            .app_data(content)
            .route("/{tail:.*}", web::get().to(index)) // match all paths starting with '/'
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
