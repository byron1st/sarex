use cmd::init_app;
use env_logger::Env;

mod cmd;
mod config;
mod model;

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    init_app().await;
}
