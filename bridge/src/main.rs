//! Hugo Phone — DTMF → Keyboard bridge.
//!
//! Se conecta al AMI de Asterisk, escucha eventos `UserEvent(HugoKey,Key=N)`
//! y los traduce a pulsaciones del teclado virtual (uinput).

use std::path::PathBuf;
use std::sync::Arc;

use clap::Parser;
use tokio::signal;
use tokio::sync::watch;
use tracing::{error, info};

mod ami;
mod config;
mod keyboard;
mod watcher;

use config::Config;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, BoxError>;

#[derive(Parser, Debug)]
#[command(version, about = "Hugo Phone DTMF → Keyboard bridge")]
struct Args {
    /// Ruta al archivo de configuración TOML.
    #[arg(short, long, default_value = "/opt/hugo-phone/bridge/config.toml")]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let args = Args::parse();

    // Carga inicial de config.
    let initial_config = Config::load(&args.config)?;
    info!("Config cargado desde {:?}", args.config);

    // Canal watch para hot-reload: cuando cambia el archivo, se envía la nueva config
    // y el loop AMI la levanta sin restart.
    let (cfg_tx, cfg_rx) = watch::channel(Arc::new(initial_config));

    // Tarea de watcher de archivo (hot-reload).
    let watcher_handle = tokio::spawn(watcher::watch_config(args.config.clone(), cfg_tx));

    // Tarea principal: conexión AMI + loop de eventos.
    let ami_handle = tokio::spawn(ami::run(cfg_rx));

    // Esperar Ctrl+C para shutdown limpio.
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("SIGINT recibido, cerrando...");
        }
        res = ami_handle => {
            if let Err(e) = res {
                error!("Tarea AMI terminó con error: {:?}", e);
            }
        }
        res = watcher_handle => {
            if let Err(e) = res {
                error!("Watcher de config terminó con error: {:?}", e);
            }
        }
    }

    Ok(())
}
