//! Cliente AMI (Asterisk Manager Interface) mínimo.
//!
//! Protocolo AMI:
//! - TCP plano
//! - Banner inicial: "Asterisk Call Manager/X.Y.Z\r\n"
//! - Mensajes: pares "Key: Value\r\n", terminados por línea vacía "\r\n"
//! - Login se hace con Action: Login, parámetros Username y Secret.
//! - Eventos vienen como Event: <Nombre> con campos adicionales.
//!
//! No usamos ActionID porque solo escuchamos eventos, no esperamos respuestas
//! específicas (excepto el login).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::keyboard::VirtualKeyboard;
use crate::{BoxError, Result};

/// Tarea principal: mantiene conexión AMI con reconexión automática.
pub async fn run(mut cfg_rx: watch::Receiver<Arc<Config>>) -> Result<()> {
    // Snapshot inicial.
    let initial_cfg = cfg_rx.borrow().clone();

    // Creamos el teclado UNA SOLA VEZ (recrearlo entre cambios de config es feo
    // y arriesga eventos perdidos). Los cambios de keymap se aplican en caliente.
    let mut keyboard = VirtualKeyboard::new(
        &initial_cfg.keymap,
        initial_cfg.bridge.press_duration_ms,
    )?;
    info!("Teclado virtual creado");

    let mut backoff_secs = 1u64;

    loop {
        let cfg = cfg_rx.borrow().clone();

        match connect_and_loop(&cfg, &mut keyboard, &mut cfg_rx).await {
            Ok(()) => {
                info!("Conexión AMI cerrada limpiamente, reconectando...");
                backoff_secs = 1;
            }
            Err(e) => {
                error!("Error en conexión AMI: {}. Reintentando en {}s...", e, backoff_secs);
                sleep(Duration::from_secs(backoff_secs)).await;
                backoff_secs = (backoff_secs * 2).min(30);
            }
        }
    }
}

async fn connect_and_loop(
    cfg: &Config,
    keyboard: &mut VirtualKeyboard,
    cfg_rx: &mut watch::Receiver<Arc<Config>>,
) -> Result<()> {
    let addr = format!("{}:{}", cfg.ami.host, cfg.ami.port);
    info!("Conectando a AMI en {} ...", addr);

    let stream = TcpStream::connect(&addr).await?;
    stream.set_nodelay(true)?;
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    // Leer banner.
    let mut banner = String::new();
    reader.read_line(&mut banner).await?;
    info!("AMI banner: {}", banner.trim());

    // Login.
    let login = format!(
        "Action: Login\r\nUsername: {}\r\nSecret: {}\r\nEvents: user,call\r\n\r\n",
        cfg.ami.username, cfg.ami.secret
    );
    write_half.write_all(login.as_bytes()).await?;
    write_half.flush().await?;

    // Esperar respuesta de login (primer mensaje completo).
    let login_resp = read_message(&mut reader).await?;
    let success = login_resp.get("Response").map(|s| s.as_str()) == Some("Success");
    if !success {
        let reason = login_resp
            .get("Message")
            .cloned()
            .unwrap_or_else(|| "razón desconocida".into());
        return Err(format!("Login AMI falló: {}", reason).into());
    }
    info!("Autenticado en AMI como {}", cfg.ami.username);

    let event_name = cfg.bridge.event_name.clone();
    let key_field = cfg.bridge.key_field.clone();

    // Loop: lee mensajes y reacciona. Atento también a cambios de config.
    loop {
        tokio::select! {
            msg = read_message(&mut reader) => {
                let msg = msg?;
                handle_event(&msg, &event_name, &key_field, keyboard).await;
            }
            changed = cfg_rx.changed() => {
                if changed.is_err() {
                    // El sender se cerró → shutdown.
                    return Ok(());
                }
                let new_cfg = cfg_rx.borrow().clone();
                info!("Recargando config en caliente");
                keyboard.update_keymap(&new_cfg.keymap);
                keyboard.set_press_duration(new_cfg.bridge.press_duration_ms);
                // Nota: cambios en ami.* requieren reconexión, no los aplicamos
                // hot — pero forzamos reconexión retornando Ok si cambió la auth.
                if new_cfg.ami.host != cfg.ami.host
                    || new_cfg.ami.port != cfg.ami.port
                    || new_cfg.ami.username != cfg.ami.username
                    || new_cfg.ami.secret != cfg.ami.secret
                {
                    info!("Config AMI cambió, reconectando...");
                    return Ok(());
                }
            }
        }
    }
}

async fn handle_event(
    msg: &HashMap<String, String>,
    event_name: &str,
    key_field: &str,
    keyboard: &mut VirtualKeyboard,
) {
    // Solo nos interesa Event == UserEvent con UserEvent == HugoKey.
    let Some(evt) = msg.get("Event") else { return };
    if evt != "UserEvent" {
        return;
    }
    // UserEvent name viene en el campo "UserEvent".
    let Some(user_evt) = msg.get("UserEvent") else { return };
    if user_evt != event_name {
        return;
    }

    let Some(dtmf) = msg.get(key_field) else {
        warn!("UserEvent {} sin campo {}", event_name, key_field);
        return;
    };

    let caller = msg
        .get("Caller")
        .map(String::as_str)
        .unwrap_or("?");
    info!("DTMF {} de {}", dtmf, caller);

    if let Err(e) = keyboard.press_dtmf(dtmf).await {
        error!("Fallo al pulsar tecla: {}", e);
    }
}

/// Lee un mensaje AMI completo (terminado por línea vacía).
async fn read_message<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> std::result::Result<HashMap<String, String>, BoxError> {
    let mut map = HashMap::new();
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            return Err("Conexión AMI cerrada por el servidor".into());
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            // Fin de mensaje.
            if map.is_empty() {
                // Línea vacía sin contenido previo, ignorar y seguir.
                continue;
            }
            debug!("AMI msg: {:?}", map);
            return Ok(map);
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
        // Líneas sin ':' se descartan (no son válidas AMI).
    }
}
