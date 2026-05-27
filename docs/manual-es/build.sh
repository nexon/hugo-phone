#!/bin/bash
# Compila el manual Hugo Phone a PDF.
# Requiere: xelatex (TeX Live full o MacTeX en macOS)

set -e
cd "$(dirname "$0")"

echo "==> Pasada 1/3 (estructura)..."
xelatex -interaction=nonstopmode main.tex > /dev/null

echo "==> Pasada 2/3 (TOC + referencias)..."
xelatex -interaction=nonstopmode main.tex > /dev/null

echo "==> Pasada 3/3 (links finales)..."
xelatex -interaction=nonstopmode main.tex > /dev/null

echo ""
echo "Manual compilado: $(pwd)/main.pdf"
ls -lh main.pdf
