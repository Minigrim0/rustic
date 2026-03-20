#!/usr/bin/env bash
set -euo pipefail

SITE="$(git rev-parse --show-toplevel)/site"

echo "==> Building site into $SITE"
mkdir -p "$SITE/doc/rust" "$SITE/doc/python"

echo "==> Copying portal homepage"
cp "$(git rev-parse --show-toplevel)/.github/pages/index.html" "$SITE/index.html"

echo "==> Building Rust docs"
RUSTDOCFLAGS="--default-theme ayu" cargo doc --no-deps
cp -r target/doc/. "$SITE/doc/rust/"

echo "==> Building Python docs"
pushd rustic-py >/dev/null
uv run maturin develop -q && uv run mkdocs build -f mkdocs.yml -d "$SITE/doc/python/" \
  || echo "    [skip] mkdocs build failed — skipping python docs"
popd >/dev/null

echo ""
echo "==> Site ready. Open http://localhost:8080"
python -m http.server 8080 --directory "$SITE"
