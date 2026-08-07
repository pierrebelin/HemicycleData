#!/usr/bin/env bash
#
# Ingestion quotidienne : registre des acteurs, scrutins, dossiers, extraction
# des textes debattus. Tourne sur le VPS, appelle le backend en boucle locale
# avec le jeton du jour.
#
# Installation (utilisateur `hemicycle`, `crontab -e`) :
#
#   17 4 * * * /home/hemicycle/app/deploy/cron/hemicycle-ingest.sh >> /home/hemicycle/shared/ingest.log 2>&1
#
# 04 h 17 UTC : loin de minuit, donc jamais a cheval sur le changement de
# jeton, et hors des heures de publication de l'Assemblee nationale.
#
# `POST /api/themes/propose` n'est **pas** appele ici : il consomme la cle
# Anthropic. La proposition reste une action deliberee de l'operateur.

set -uo pipefail

APP_DIR="${HEMICYCLE_APP_DIR:-$HOME/app}"
API="${HEMICYCLE_API:-http://127.0.0.1:8085}"

TOKEN="$("$APP_DIR/deploy/bin/admin-token.sh")" || {
    echo "$(date -u +%FT%TZ) — jeton indérivable, ingestion abandonnée" >&2
    exit 1
}

# Ordre impose par les dependances : sans acteurs a jour, un scrutin reference
# des deputes inconnus ; sans scrutins, l'extraction des textes ne voit rien.
ROUTES=(
    /api/registry/refresh
    /api/scrutins/refresh
    /api/refresh
    /api/themes/extract
)

failures=0

for route in "${ROUTES[@]}"; do
    status="$(curl --silent --show-error --output /tmp/hemicycle-ingest.body \
        --write-out '%{http_code}' --max-time 600 \
        --request POST "$API$route" \
        --header "x-admin-token: $TOKEN" \
        --header 'content-type: application/json' \
        --data '{}')"

    if [[ "$status" == 2* ]]; then
        echo "$(date -u +%FT%TZ) — $route : $status"
    else
        # Le corps peut contenir un message d'erreur applicatif ; jamais le
        # jeton, qui ne voyage que dans l'en-tete.
        echo "$(date -u +%FT%TZ) — $route : $status — $(head -c 500 /tmp/hemicycle-ingest.body)" >&2
        failures=$((failures + 1))
    fi
done

rm -f /tmp/hemicycle-ingest.body

# Une route en echec ne doit pas empecher les suivantes de tourner, mais le
# code de sortie doit le refleter — sinon l'echec passe inapercu.
exit $((failures > 0 ? 1 : 0))
