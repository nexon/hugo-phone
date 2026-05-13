# Hugo Phone

Sistema offline para jugar Hugo (DOS) con un teléfono analógico antiguo vía DTMF, replicando la experiencia TV de los 90. Todo corre en una Raspberry Pi 4 conectada por HDMI a un TV.

## Stack

- **Raspberry Pi 4 (2GB)** con Raspberry Pi OS Lite 64-bit
- **Asterisk 20** como PBX (decodificación DTMF + dialplan)
- **Bridge en Rust** (binario nativo ARM, sin runtime) que escucha AMI e inyecta teclas vía `/dev/uinput`
- **DOSBox-Staging** corriendo Hugo, salida HDMI
- **Grandstream HT801** (ATA, 1 puerto FXS) + teléfono analógico DTMF

## Arquitectura

```
[Teléfono DTMF] → RJ11 → [HT801] → Ethernet ┐
                                            │
                                            ▼
                            ┌────────────────────────────────┐
                            │  Raspberry Pi 4                │
                            │  ├── Asterisk (PBX, AMI)       │
                            │  ├── hugo-bridge (Rust)        │
                            │  │     AMI → /dev/uinput       │
                            │  └── DOSBox-Staging + Hugo     │
                            └────────────┬───────────────────┘
                                         │ HDMI
                                         ▼
                                       [TV]
```

## Mapeo DTMF → Teclado

| DTMF | Tecla | Acción Hugo |
|------|-------|-------------|
| 2    | ↑     | Saltar / subir |
| 4    | ←     | Izquierda |
| 6    | →     | Derecha |
| 8    | ↓     | Bajar |
| 5    | Space | Acción |
| 0    | Esc   | Pausa |

Configurable en `bridge/config.toml` (hot-reload sin restart).

## Fases

1. [Fase 1: Preparar la Pi e instalar Asterisk](docs/fase1-pi-asterisk.md)
2. [Fase 2: Extensión SIP, probar con softphone](docs/fase2-sip-softphone.md)
3. [Fase 3: DOSBox-Staging + Hugo](docs/fase3-dosbox-hugo.md)
4. [Fase 4: Bridge Rust DTMF → teclado](docs/fase4-bridge-rust.md)
5. [Fase 5: Conectar HT801 + teléfono real](docs/fase5-ht801.md)
6. [Fase 6 (futuro): Multi-jugador](docs/fase6-multijugador.md)

## Estructura

```
hugo-phone/
├── README.md
├── asterisk/                  # /etc/asterisk/
│   ├── sip.conf
│   ├── extensions.conf
│   └── manager.conf
├── bridge/                    # Rust workspace
│   ├── Cargo.toml
│   ├── config.toml            # config en runtime (hot-reload)
│   ├── hugo-bridge.service    # systemd unit
│   └── src/
│       ├── main.rs
│       ├── config.rs          # parseo TOML + serde
│       ├── ami.rs             # cliente AMI minimal (~150 LOC)
│       ├── keyboard.rs        # teclado virtual uinput
│       └── watcher.rs         # file watcher para hot-reload
├── dosbox/
│   └── dosbox-hugo.conf
├── scripts/
│   ├── install-all.sh
│   └── download-hugo.sh
└── docs/                      # guías paso a paso
```

## Por qué Rust

- **Latencia menor** que Python: la cadena DTMF → tecla es crítica para la jugabilidad
- **Sin runtime ni deps de pip:** un binario y listo
- **Menor consumo de RAM** (crítico en Pi 2GB; DOSBox quiere todo lo que pueda)
- **Crash-resistant:** servicio 24/7

## Inicio rápido

En la Pi después de SSH:
```bash
git clone <repo> hugo-phone
cd hugo-phone
sudo bash scripts/install-all.sh
```

El script instala Asterisk, DOSBox-Staging, Rust toolchain, compila el bridge (~5-8 min), configura systemd y deja todo cableado.

Luego seguir las guías por fase para terminar de configurar y testear.

## Hot-reload del bridge

Edita `/opt/hugo-phone/bridge/config.toml` y guarda. El bridge detecta el cambio y aplica:
- Cambios en `[keymap]` → instantáneo
- Cambios en `press_duration_ms` → instantáneo
- Cambios en `[ami]` (host/port/auth) → fuerza reconexión

Si pones un valor inválido, el bridge loguea un warning y mantiene el config anterior funcionando.
