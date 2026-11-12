#!/bin/sh
# SmartCAD: compila la web y la publica en R2 (servida por el Worker smartcad-web).
#
#   sh deploy/cloudflare/publish.sh                  # compila (perfil preview) y publica
#   PROFILE=release sh deploy/cloudflare/publish.sh  # compilación optimizada (producción)
#   SKIP_BUILD=1 sh deploy/cloudflare/publish.sh     # solo publica (p. ej. tras editar
#                                                    # web/smartcad-config.js)
#
# Solo sube los archivos que cambiaron desde la última publicación
# (manifiesto en dist/.published.sha256).
set -eu
cd "$(dirname "$0")/../.."
export PATH="$HOME/.cargo/bin:$PATH"
export CLOUDFLARE_ACCOUNT_ID=f284cf099d76922cf6981a8425350f7b
BUCKET=smartcad
PROFILE=${PROFILE:-preview}
MANIFEST=dist/.published.sha256

if [ -z "${SKIP_BUILD:-}" ]; then
  if [ "$PROFILE" = release ]; then
    trunk build --locked --release --public-url / --dist dist/app \
      --html-output index.html web-app.html
  else
    trunk build --locked --cargo-profile "$PROFILE" --public-url / --dist dist/app \
      --html-output index.html web-app.html
  fi
else
  # Sin compilar: copia la configuración web por si se editó.
  cp web/smartcad-config.js dist/app/smartcad-config.js
fi

cd dist/app
hash=$(find . -type f ! -name sw.js ! -name .trunk-lock -print0 | sort -z \
  | xargs -0 shasum -a 256 | shasum -a 256 | cut -c1-12)
sed -i '' "s/^const CACHE_VERSION = \"[^\"]*\";/const CACHE_VERSION = \"$hash\";/" sw.js
echo '[]' > supporters.json
printf '{"product":"SmartCAD","commit":"%s","source":"https://github.com/mkJou/smartcad"}\n' \
  "$(git rev-parse HEAD)" > release.json

touch "../../$MANIFEST"
current=$(mktemp)
find . -type f ! -name .trunk-lock ! -name generate.sh | sed 's|^\./||' | sort \
  | while read -r f; do printf '%s  %s\n' "$(shasum -a 256 "$f" | cut -c1-64)" "$f"; done > "$current"

# Cambiados = líneas nuevas respecto al manifiesto anterior.
changed=$(comm -13 "../../$MANIFEST" "$current" | sed 's/^[0-9a-f]*  //')
n=0
# Primero todo menos index.html/sw.js; esos al final (nunca un index que
# apunte a un wasm que aún no está).
for f in $(printf '%s\n' "$changed" | grep -vE '^(index\.html|sw\.js)$' || true) \
         $(printf '%s\n' "$changed" | grep -E '^(sw\.js|index\.html)$' | sort -r || true); do
  [ -n "$f" ] || continue
  wrangler r2 object put "$BUCKET/$f" --file "$f" --remote >/dev/null
  echo "  subido $f"
  n=$((n + 1))
done
mv "$current" "../../$MANIFEST"
echo "Publicado ($n archivos): https://cad-preview.serviciosmaracaibo.com  [bundle $hash, perfil $PROFILE]"
