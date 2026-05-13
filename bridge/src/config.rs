//! Carga y parseo del archivo de configuración TOML.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub ami: AmiConfig,
    #[serde(default)]
    pub bridge: BridgeConfig,
    #[serde(default = "default_keymap")]
    pub keymap: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AmiConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BridgeConfig {
    /// Duración de cada "tap" del teclado en milisegundos.
    #[serde(default = "default_press_ms")]
    pub press_duration_ms: u64,
    /// Nombre del evento UserEvent que escuchamos.
    #[serde(default = "default_event_name")]
    pub event_name: String,
    /// Nombre del campo del UserEvent que trae la tecla.
    #[serde(default = "default_key_field")]
    pub key_field: String,
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            press_duration_ms: default_press_ms(),
            event_name: default_event_name(),
            key_field: default_key_field(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    5038
}
fn default_press_ms() -> u64 {
    80
}
fn default_event_name() -> String {
    "HugoKey".into()
}
fn default_key_field() -> String {
    "Key".into()
}

/// Mapeo DTMF → nombre simbólico de tecla (resuelto luego en keyboard.rs).
fn default_keymap() -> HashMap<String, String> {
    [
        ("2", "Up"),
        ("4", "Left"),
        ("6", "Right"),
        ("8", "Down"),
        ("5", "Space"),
        ("0", "Esc"),
        ("1", "Q"),
        ("3", "E"),
        ("7", "Z"),
        ("9", "X"),
        ("*", "LeftShift"),
        ("#", "Enter"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let cfg: Config = toml::from_str(&content)?;
        Ok(cfg)
    }
}
