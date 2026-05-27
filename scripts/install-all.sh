#!/bin/bash
# scripts/install-all.sh
# Hugo Phone — Instalación completa en Raspberry Pi 4
#
# Uso: sudo bash scripts/install-all.sh
#
# Instala: Asterisk, DOSBox, Rust toolchain, compila el bridge,
# configura servicio systemd, copia configs.

set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "Debes correr como root (sudo)."
  exit 1
fi

REPO_DIR="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_DIR="/opt/hugo-phone"
BUILD_USER="$(getent passwd 1000 | cut -d: -f1)"
BUILD_HOME="$(getent passwd 1000 | cut -d: -f6)"

echo "==> Hugo Phone install desde $REPO_DIR"
echo "==> Usuario de build: $BUILD_USER"

# ------------------------------------------------------------------
# 1. Paquetes del sistema
# ------------------------------------------------------------------
echo "==> Actualizando apt..."
apt update
apt upgrade -y

#echo "==> Instalando paquetes base..."
#apt install -y \
#  asterisk asterisk-modules \
#  dosbox \
#  build-essential pkg-config \
#  libudev-dev \
#  xserver-xorg xinit openbox \
#  alsa-utils \
#  git curl unzip

#echo "==> Instalando paquetes de dosbox.."
#apt install -y \
#   xserver-xorg xinit openbox \
#   dosbox \
#   alsa-utils

# ------------------------------------------------------------------
# 2. Asterisk configs (Asterisk 22 — chan_pjsip, ya no chan_sip)
# ------------------------------------------------------------------
echo "==> Copiando configs Asterisk..."
cp "$REPO_DIR/config/asterisk/pjsip.conf"      /etc/asterisk/pjsip.conf
cp "$REPO_DIR/config/asterisk/extensions.conf" /etc/asterisk/extensions.conf
cp "$REPO_DIR/config/asterisk/manager.conf"    /etc/asterisk/manager.conf
chown asterisk:asterisk /etc/asterisk/*.conf
chmod 640 /etc/asterisk/*.conf

systemctl enable asterisk
systemctl restart asterisk

# ------------------------------------------------------------------
# 3. Instalar Rust toolchain (como usuario, no root)
# ------------------------------------------------------------------
if ! sudo -u "$BUILD_USER" bash -c 'command -v cargo' >/dev/null 2>&1; then
  echo "==> Instalando Rust toolchain (vía rustup)..."
  sudo -u "$BUILD_USER" bash -c \
    'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal'
else
  echo "==> Rust ya instalado."
fi

CARGO="$BUILD_HOME/.cargo/bin/cargo"

# ------------------------------------------------------------------
# 4. Compilar el bridge
# ------------------------------------------------------------------
echo "==> Compilando hugo-bridge (esto toma ~5-8 min en Pi 4)..."
mkdir -p "$INSTALL_DIR"
cp -r "$REPO_DIR/bridge" "$INSTALL_DIR/"
chown -R "$BUILD_USER:$BUILD_USER" "$INSTALL_DIR/bridge"

sudo -u "$BUILD_USER" bash -c "cd $INSTALL_DIR/bridge && $CARGO build --release"

cp "$INSTALL_DIR/bridge/target/release/hugo-bridge" "$INSTALL_DIR/bridge/hugo-bridge"
chmod +x "$INSTALL_DIR/bridge/hugo-bridge"

# ------------------------------------------------------------------
# 5. uinput: módulo + permisos
# ------------------------------------------------------------------
if ! getent group input >/dev/null; then
  groupadd input
fi

cat > /etc/udev/rules.d/99-hugo-uinput.rules <<'EOF'
KERNEL=="uinput", GROUP="input", MODE="0660", OPTIONS+="static_node=uinput"
EOF
udevadm control --reload-rules
udevadm trigger

if ! grep -q '^uinput' /etc/modules; then
  echo "uinput" >> /etc/modules
fi
modprobe uinput || true

# ------------------------------------------------------------------
# 6. Servicio systemd
# ------------------------------------------------------------------
cp "$REPO_DIR/bridge/hugo-bridge.service" /etc/systemd/system/hugo-bridge.service
systemctl daemon-reload
systemctl enable hugo-bridge

# ------------------------------------------------------------------
# 7. Config DOSBox
# ------------------------------------------------------------------
# El config vive en /opt/hugo-phone/games/ y se invoca con `dosbox -conf …`.
# Así evitamos depender del path por versión de DOSBox (~/.dosbox/dosbox-X.Y.Z.conf).
mkdir -p "$INSTALL_DIR/games"
cp "$REPO_DIR/config/dosbox/dosbox-hugo.conf" "$INSTALL_DIR/games/dosbox-hugo.conf"
chown -R "$BUILD_USER:$BUILD_USER" "$INSTALL_DIR/games"

echo ""
echo "============================================================"
echo "  Instalación completa."
echo "============================================================"
echo ""
echo "Pasos siguientes:"
echo "  1. Editar secretos (que coincidan entre sí):"
echo "       sudo nano /etc/asterisk/pjsip.conf"
echo "       sudo nano /etc/asterisk/manager.conf"
echo "       sudo nano $INSTALL_DIR/bridge/config.toml"
echo "  2. sudo systemctl restart asterisk"
echo "  3. Descargar Hugo a $INSTALL_DIR/games/hugo/"
echo "  4. sudo systemctl start hugo-bridge"
echo "  5. sudo journalctl -u hugo-bridge -f"
echo ""
echo "Ver docs/fase1-pi-asterisk.md para guía detallada."
