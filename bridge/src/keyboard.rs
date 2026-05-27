//! Teclado virtual vía /dev/uinput.
//!
//! Crea un device input que el kernel ve como un teclado físico. DOSBox
//! lo recibe vía SDL/evdev como cualquier teclado USB.
//!
//! Requiere permisos sobre /dev/uinput (correr como root o con regla udev).

use std::collections::HashMap;
use std::time::Duration;

use tokio::time::sleep;
use tracing::{debug, warn};
use uinput::event::keyboard::Key;

use crate::Result;

pub struct VirtualKeyboard {
    device: uinput::Device,
    keymap: HashMap<String, Key>,
    press_duration: Duration,
}

impl VirtualKeyboard {
    /// Crea el teclado virtual registrando todas las keys (`Keyboard::All`),
    /// para que cualquier cambio de keymap en caliente funcione sin recrear.
    pub fn new(
        symbolic_map: &HashMap<String, String>,
        press_duration_ms: u64,
    ) -> Result<Self> {
        let keymap = resolve_keymap(symbolic_map);

        let device = uinput::default()?
            .name("hugo-phone-virtual-keyboard")?
            .event(uinput::event::Keyboard::All)?
            .create()?;

        Ok(Self {
            device,
            keymap,
            press_duration: Duration::from_millis(press_duration_ms),
        })
    }

    /// Pulsa la tecla correspondiente al DTMF. No-op si no hay mapeo.
    pub async fn press_dtmf(&mut self, dtmf: &str) -> Result<()> {
        let Some(key) = self.keymap.get(dtmf) else {
            warn!("DTMF {:?} sin mapeo", dtmf);
            return Ok(());
        };

        debug!("Pulsando DTMF {} → {:?}", dtmf, key);

        self.device.press(key)?;
        self.device.synchronize()?;
        sleep(self.press_duration).await;
        self.device.release(key)?;
        self.device.synchronize()?;

        Ok(())
    }

    /// Actualiza el keymap en caliente.
    pub fn update_keymap(&mut self, new_symbolic: &HashMap<String, String>) {
        self.keymap = resolve_keymap(new_symbolic);
    }

    pub fn set_press_duration(&mut self, ms: u64) {
        self.press_duration = Duration::from_millis(ms);
    }
}

fn resolve_keymap(symbolic: &HashMap<String, String>) -> HashMap<String, Key> {
    let mut out = HashMap::with_capacity(symbolic.len());
    for (dtmf, name) in symbolic {
        match parse_key_name(name) {
            Some(k) => {
                out.insert(dtmf.clone(), k);
            }
            None => {
                warn!("Tecla desconocida en config: {:?} (DTMF {:?})", name, dtmf);
            }
        }
    }
    out
}

/// Nombre simbólico → `uinput::event::keyboard::Key`.
fn parse_key_name(name: &str) -> Option<Key> {
    Some(match name {
        "Up" => Key::Up,
        "Down" => Key::Down,
        "Left" => Key::Left,
        "Right" => Key::Right,
        "Space" => Key::Space,
        "Enter" => Key::Enter,
        "Esc" | "Escape" => Key::Esc,
        "Tab" => Key::Tab,
        "Backspace" => Key::BackSpace,
        "LeftShift" => Key::LeftShift,
        "RightShift" => Key::RightShift,
        "LeftCtrl" => Key::LeftControl,
        "RightCtrl" => Key::RightControl,
        "LeftAlt" => Key::LeftAlt,
        "RightAlt" => Key::RightAlt,
        // Letras
        "A" => Key::A, "B" => Key::B, "C" => Key::C, "D" => Key::D, "E" => Key::E,
        "F" => Key::F, "G" => Key::G, "H" => Key::H, "I" => Key::I, "J" => Key::J,
        "K" => Key::K, "L" => Key::L, "M" => Key::M, "N" => Key::N, "O" => Key::O,
        "P" => Key::P, "Q" => Key::Q, "R" => Key::R, "S" => Key::S, "T" => Key::T,
        "U" => Key::U, "V" => Key::V, "W" => Key::W, "X" => Key::X, "Y" => Key::Y,
        "Z" => Key::Z,
        // Función
        "F1" => Key::F1, "F2" => Key::F2, "F3" => Key::F3, "F4" => Key::F4,
        "F5" => Key::F5, "F6" => Key::F6, "F7" => Key::F7, "F8" => Key::F8,
        "F9" => Key::F9, "F10" => Key::F10, "F11" => Key::F11, "F12" => Key::F12,
        _ => return None,
    })
}
