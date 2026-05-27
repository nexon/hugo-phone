# Manual Hugo Phone — Fuentes LaTeX

Este directorio contiene las fuentes LaTeX del manual técnico de Hugo Phone
y el PDF ya compilado.

## Archivos

- `main.tex` — Documento maestro (preámbulo, portada, includes)
- `cap01-introduccion.tex` ... `cap10-referencia.tex` — Capítulos
- `main.pdf` — PDF compilado (49 páginas, ~280 KB)
- `build.sh` — Script para recompilar

## Recompilar

Requiere TeX Live o MacTeX (en macOS: `brew install --cask mactex`).

```bash
bash build.sh
```

El script corre xelatex tres veces para que el TOC y los hyperlinks
queden bien resueltos.

## Paquetes LaTeX usados

Todos vienen en una instalación completa de TeX Live / MacTeX:

- fontspec, babel (con shorthands desactivados)
- geometry, parskip, setspace
- xcolor, tcolorbox (para cajas Nota/Advertencia/Tip)
- fancyhdr, titlesec (encabezados y capítulos)
- listings (syntax highlighting Rust, TOML, Asterisk, shell)
- tikz (diagramas de arquitectura)
- booktabs, longtable, tabularx (tablas)
- hyperref (links internos y externos)

## Editar

Cada capítulo es un archivo independiente; modifica el que corresponda
y recompila. La numeración de capítulos y referencias cruzadas se
actualiza sola.
