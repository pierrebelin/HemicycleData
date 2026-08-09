#!/usr/bin/env bash
#
# Ingestion periodique : registre des acteurs, scrutins, dossiers, extraction
# des textes debattus et rattachement thematique. Tourne sur le VPS, appelle le
# backend en boucle locale avec le jeton du jour.
#
# Declenchement par `hemicycle-ingest.timer` (toutes les deux heures), installe
# en meme temps que les autres unites systemd. Voir todo/SPEC-DEPLOIEMENT.md
# §3.3 : le timer et une ligne de crontab s'excluent, il ne faut pas les deux.
#
# Repli si l'on prefere la crontab au timer (utilisateur `hemicycle`,
# `crontab -e`), a n'installer que si le timer ne l'est pas :
#
#   17 4 * * * /home/hemicycle/app/deploy/cron/hemicycle-ingest.sh >> /home/hemicycle/shared/ingest.log 2>&1
#
# Le jeton du jour et celui de la veille sont acceptes tous les deux : aucune
# cadence ne tombe donc a cheval sur le changement de jeton de minuit.
#
# `POST /api/refresh` **consomme la cle Anthropic** : depuis le 9 aout 2026 il
# extrait les textes debattus puis rattache les objets encore en attente. Le
# volume est plafonne par `THEME_BATCH_PER_REFRESH` (100 par defaut), et un
# objet deja rattache n'est jamais resoumis : une passe de routine ne paie que
# ce qui est nouveau. Poser `THEME_BATCH_PER_REFRESH=0` dans l'environnement du
# service suspend le rattachement sans toucher au reste de l'ingestion.
#
# `POST /api/themes/propose` reste disponible pour rattraper un arriere a la
# main, hors cadence.
#
# `POST /api/amendements/refresh` ingere les amendements. Deux garde-fous, sans
# lesquels une passe deborderait la fenetre du timer : l'archive republiee a
# l'identique est reconnue par son ETag et n'est pas reparcourue, et le volume
# ecrit par passe est plafonne par `AMENDMENT_BATCH_PER_REFRESH` (40 000 par
# defaut). Le premier chargement complet s'etale donc sur plusieurs passes ; le
# lancer a la main avec `AMENDMENT_BATCH_PER_REFRESH=0` evite d'occuper la
# cadence pendant une journee.

set -uo pipefail

APP_DIR="${HEMICYCLE_APP_DIR:-$HOME/app}"
API="${HEMICYCLE_API:-http://127.0.0.1:8085}"
HEALTH_RETRIES=30

# Le timer suit le demarrage du service : les migrations sqlx et le reveil a
# froid de Neon peuvent encore etre en cours. Attendre plutot que de compter
# quatre echecs sur une simple fenetre de demarrage.
for i in $(seq 1 "$HEALTH_RETRIES"); do
    if curl --fail --silent --show-error --max-time 5 "$API/api/health" >/dev/null 2>&1; then
        break
    fi
    if [[ "$i" -eq "$HEALTH_RETRIES" ]]; then
        echo "$(date -u +%FT%TZ) — $API/api/health muet apres ${HEALTH_RETRIES}s, ingestion abandonnée" >&2
        exit 1
    fi
    sleep 1
done

TOKEN="$("$APP_DIR/deploy/bin/admin-token.sh")" || {
    echo "$(date -u +%FT%TZ) — jeton indérivable, ingestion abandonnée" >&2
    exit 1
}

# Ordre impose par les dependances : sans acteurs a jour, un scrutin reference
# des deputes inconnus ; sans scrutins, l'extraction des textes ne voit rien.
# `/api/refresh` ferme la marche et porte l'extraction et le rattachement : les
# appeler separement ici les rejouerait a vide.
ROUTES=(
    /api/registry/refresh
    /api/scrutins/refresh
    /api/amendements/refresh
    /api/refresh
)

failures=0

for route in "${ROUTES[@]}"; do
    status="$(curl --silent --show-error --output /tmp/hemicycle-ingest.body \
        --write-out '%{http_code}' --max-time 900 \
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
