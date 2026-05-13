//! Watcher del archivo de configuración: recarga en caliente cuando cambia.
//!
//! Usa `notify` para detectar writes/replaces. Debounce de 500ms para evitar
//! recargar varias veces si el editor escribe en múltiples pasos.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notify::{Event, EventKind, RecursiveMode, Watcher};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use tracing::{error, info, warn};

use crate::config::Config;
use crate::Result;

pub async fn watch_config(
    path: PathBuf,
    cfg_tx: watch::Sender<Arc<Config>>,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();

    // El watcher de `notify` no es async; lo movemos a un thread y los eventos
    // los pasamos por un mpsc al runtime de tokio.
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        match res {
            Ok(evt) => {
                let _ = tx.send(evt);
            }
            Err(e) => error!("Watcher error: {}", e),
        }
    })?;

    // Watch sobre el directorio padre (más confiable que sobre el archivo,
    // porque muchos editores hacen "replace" en vez de "modify").
    let parent = path
        .parent()
        .ok_or("config path no tiene directorio padre")?;
    watcher.watch(parent, RecursiveMode::NonRecursive)?;
    info!("Watching {:?} para hot-reload", path);

    let mut pending_reload = false;

    loop {
        tokio::select! {
            evt = rx.recv() => {
                let Some(evt) = evt else {
                    warn!("Watcher channel cerrado");
                    return Ok(());
                };

                if !matches!(
                    evt.kind,
                    EventKind::Modify(_) | EventKind::Create(_)
                ) {
                    continue;
                }

                if !evt.paths.iter().any(|p| p == &path) {
                    continue;
                }

                pending_reload = true;
            }
            _ = sleep(Duration::from_millis(500)), if pending_reload => {
                pending_reload = false;
                match Config::load(&path) {
                    Ok(new_cfg) => {
                        info!("Config recargado");
                        if cfg_tx.send(Arc::new(new_cfg)).is_err() {
                            // Receiver dropped → shutdown.
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        error!("Error al recargar config: {}. Manteniendo config anterior.", e);
                    }
                }
            }
        }
    }
}
