#!/usr/bin/env bash
#
# Capture quotidienne des versions officielles de textes rattachees a un vote
# final. Ce job ne passe ni par une route d'administration, ni par un LLM : il
# synchronise le registre versionne puis fige les documents Open Data dans
# PostgreSQL.
#
# Declenchement par `hemicycle-text-capture.timer`. Les binaires sont compiles
# au deploiement ; ne pas employer `cargo run` ici, l'unite systemd interdit
# les ecritures dans le repertoire de l'application.

set -euo pipefail

APP_DIR="${HEMICYCLE_APP_DIR:-$HOME/app}"
API="${HEMICYCLE_API:-http://127.0.0.1:8085}"
HEALTH_RETRIES=30

SYNC_BINARY="$APP_DIR/target/release/sync-official-text-versions"
CAPTURE_BINARY="$APP_DIR/target/release/capture-official-text-versions"

for binary in "$SYNC_BINARY" "$CAPTURE_BINARY"; do
    if [[ ! -x "$binary" ]]; then
        echo "$(date -u +%FT%TZ) — binaire absent ou non executable : $binary" >&2
        exit 1
    fi
done

# Le service principal execute les migrations avant de repondre au health
# check. Attendre cette reponse evite que le tout premier passage cherche des
# tables encore absentes juste apres un deploiement.
for i in $(seq 1 "$HEALTH_RETRIES"); do
    if curl --fail --silent --show-error --max-time 5 "$API/api/health" >/dev/null 2>&1; then
        break
    fi
    if [[ "$i" -eq "$HEALTH_RETRIES" ]]; then
        echo "$(date -u +%FT%TZ) — $API/api/health muet apres ${HEALTH_RETRIES}s, capture abandonnee" >&2
        exit 1
    fi
    sleep 1
done

echo "$(date -u +%FT%TZ) — synchronisation des versions officielles"
"$SYNC_BINARY"

echo "$(date -u +%FT%TZ) — capture des documents officiels"
"$CAPTURE_BINARY"
