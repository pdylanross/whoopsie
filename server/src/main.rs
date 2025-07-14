#[macro_use]
extern crate thiserror;
#[macro_use]
extern crate tracing;

pub(crate) mod config;
pub(crate) mod db;
pub mod extensions;
mod fileserv;
mod monitor;
mod signal;

use crate::config::{MonitorGeneralConfig, ServerConfig};
use crate::db::get_db_factory;
use crate::extensions::MappingExt;
use crate::fileserv::file_and_error_handler;
use crate::monitor::MonitorController;
use crate::signal::{ExitSignal, ExitSignaler};
use app::state::ServerState;
use app::{shell, App};
use axum::Router;
use leptos::config::get_configuration;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::net::SocketAddr;
use tokio::try_join;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    app::tracing::init_tracing();

    let exit_signaler = ExitSignaler::new();

    let server_config = config::load_config()?;
    let server_state = build_server_state(server_config).await?;
    let monitor_controller = MonitorController::new(server_state.clone());

    let leptos_options = server_state.leptos_options.clone();
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&server_state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(file_and_error_handler)
        .with_state(server_state);

    let monitoring = monitor_controller.start(exit_signaler.clone());
    let http = tokio::spawn(listen_and_serve(addr, app, exit_signaler.new_exit_signal()));

    match try_join!(http, monitoring, exit_signaler.wait_for_shutdown()) {
        Ok(res) => {
            let (http_res, monitoring_res, exit_res) = res;
            let vec = [http_res, monitoring_res, exit_res];
            let errs = vec.into_iter().filter_map(|r| r.err()).collect::<Vec<_>>();

            if errs.is_empty() {
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Errors occurred during shutdown: {:?}",
                    errs
                ))
            }
        }
        Err(err) => Err(err.into()),
    }
}

async fn listen_and_serve(
    addr: SocketAddr,
    app: Router,
    exit_signal: ExitSignal,
) -> Result<(), anyhow::Error> {
    info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(exit_signal.wait_owned())
        .await?;
    Ok(())
}

fn load_env() {
    use dotenvy::*;
    dotenv().ok();

    if std::env::var("LOCAL_DEV").is_ok_and(|v| v == "true") {
        debug!("local dev mode enabled");
        from_filename(".env.local").ok();
    }
}

async fn build_server_state(scfg: ServerConfig) -> Result<ServerState, anyhow::Error> {
    let conf = get_configuration(None).expect("Failed to load leptos configuration");
    let db_factory = get_db_factory(&scfg.db).await?;

    db_factory.initialize_db().await?;

    let global_config = if let Some(cfg) = scfg.global_monitor_config {
        cfg.object_map()
    } else {
        MonitorGeneralConfig::default().object_map()
    };

    Ok(ServerState {
        leptos_options: conf.leptos_options,
        app_config: scfg.app_config.unwrap_or_default(),
        db_factory,
        global_config,
    })
}
