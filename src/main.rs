use env_logger::Env;

mod app;

#[tokio::main]
async fn main() {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);

    app::cli::init_app().await;
}
