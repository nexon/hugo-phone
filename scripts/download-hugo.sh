#!/bin/bash
# scripts/download-hugo.sh
# Hugo Phone — Descarga Hugo desde MyAbandonware
#
# Hugo (1996) por ITE Media es abandonware. Se distribuye legalmente como tal
# en sitios como MyAbandonware. Este script descarga, descomprime y deja el
# juego listo para DOSBox.
#
# Hugo 1/2/3 vienen como ZIP con HUGO.EXE suelto (sin instalación).
# Hugo 4 viene como ISO de CD-ROM y requiere correr INSTALL.EXE adentro de
# DOSBox la primera vez (ver docs/manual-es/cap06-fase3.tex).
#
# Uso: bash scripts/download-hugo.sh

set -euo pipefail

DEST="/opt/hugo-phone/games/hugo"

echo "==> Hugo Phone — descarga del juego"
echo ""
echo "NOTA: el script no puede descargar Hugo automáticamente porque"
echo "MyAbandonware requiere interacción humana (CAPTCHA / botón)."
echo ""
echo "Pasos manuales:"
echo "  1. Abre en un navegador: https://www.myabandonware.com/game/hugo-eya"
echo "  2. Descarga el ZIP del juego"
echo "  3. Copia el ZIP a la Pi:"
echo "       scp Hugo*.zip beto@hugo-pbx.local:/tmp/"
echo "  4. En la Pi, descomprime al destino:"
echo "       sudo mkdir -p $DEST"
echo "       sudo unzip /tmp/Hugo*.zip -d $DEST"
echo "       sudo chown -R \$USER:\$USER $DEST"
echo ""
echo "  5. Verifica que hugo.exe esté en $DEST/"
echo "       ls $DEST/hugo.exe"
echo ""
echo "Versiones alternativas:"
echo "  Hugo 1 (1996):  https://www.myabandonware.com/game/hugo-eya"
echo "  Hugo 3 (1995):  https://www.myabandonware.com/game/hugo-3-ey9"
echo ""
