# Hugo Phone manual — LaTeX sources

This directory contains the LaTeX sources of the Hugo Phone technical
manual along with the compiled PDF.

## Files

- `main.tex` — Master document (preamble, title page, includes)
- `cap01-introduccion.tex` ... `cap10-referencia.tex` — Chapters
- `main.pdf` — Compiled PDF (~49 pages, ~280 KB)
- `build.sh` — Rebuild script

## Rebuild

Requires TeX Live or MacTeX (on macOS: `brew install --cask mactex`).

```bash
bash build.sh
```

The script runs xelatex three times so the TOC and hyperlinks are
resolved properly.

## LaTeX packages used

All ship with a full TeX Live / MacTeX install:

- fontspec, babel
- geometry, parskip, setspace
- xcolor, tcolorbox (for Note/Warning/Tip callouts)
- fancyhdr, titlesec (headers and chapters)
- listings (syntax highlighting for Rust, TOML, Asterisk, shell)
- tikz (architecture diagrams)
- booktabs, longtable, tabularx (tables)
- hyperref (internal and external links)

## Editing

Each chapter is an independent file; edit the relevant one and
rebuild. Chapter numbering and cross-references update on their own.
