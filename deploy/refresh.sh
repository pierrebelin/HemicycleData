#!/usr/bin/env bash
#
# Rafraichissement periodique des donnees, appele par `hemicycle-refresh.timer`
# toutes les deux heures. Un seul appel :
#
#   POST /api/refresh  →  referentiel des acteurs, puis dossiers, puis scrutins
#
# L'ordre est impose cote application (RefreshAll) : rafraichir les dossiers
# avant le referentiel rattacherait les initiateurs sur des appartenances
# perimees. Le script n'a donc rien a orchestrer, il declenche.
#
# L'appel passe par la boucle locale et non par Nginx : le vhost public refuse
# les POST sous /api/ (todo/SPEC-DEPLOIEMENT.md §3.1), et le vhost admin
# n'ajouterait qu'un saut de proxy.
#
# Options :
#   --full   ajoute `?full=true` : reecrit tous les dossiers au lieu des seuls
#            qui ont bouge. A reserver a un changement de regle de derivation
#            (score, sort, rattachement), lance a la main. Pas dans le timer.

set -euo pipefail

PORT="${PORT:-8085}"
BASE_URL="http://127.0.0.1:${PORT}"
HEALTH_URL="${BASE_URL}/api/health"
HEALTH_RETRIES=30

# Une ingestion lit toute la source de l'Assemblee. Large, mais borne : au-dela,
# c'est un blocage, pas une lenteur.
MAX_TIME="${REFRESH_MAX_TIME:-1800}"

QUERY=""

for arg in "$@"; do
    case "$arg" in
        --full) QUERY="?full=true" ;;
        *)      echo "Option inconnue : $arg" >&2; exit 2 ;;
    esac
done

# Le timer suit le demarrage du service ; les migrations sqlx et le reveil a
# froid de Neon peuvent encore etre en cours. Attendre plutot qu'echouer.
for i in $(seq 1 "$HEALTH_RETRIES"); do
    if curl -fsS --max-time 5 "$HEALTH_URL" >/dev/null 2>&1; then
        break
    fi
    if [ "$i" -eq "$HEALTH_RETRIES" ]; then
        echo "ECHEC : ${HEALTH_URL} ne repond pas apres ${HEALTH_RETRIES}s" >&2
        exit 1
    fi
    sleep 1
done

BODY_FILE="$(mktemp)"
trap 'rm -f "$BODY_FILE"' EXIT

echo "Rafraichissement : POST ${BASE_URL}/api/refresh${QUERY}"
STARTED_AT=$SECONDS

if ! STATUS="$(curl -sS -o "$BODY_FILE" -w '%{http_code}' \
        --max-time "$MAX_TIME" \
        -X POST "${BASE_URL}/api/refresh${QUERY}")"; then
    echo "ECHEC : appel interrompu (timeout ${MAX_TIME}s ou reseau)" >&2
    exit 1
fi

ELAPSED=$((SECONDS - STARTED_AT))

if [ "$STATUS" != "200" ]; then
    echo "ECHEC : HTTP ${STATUS} apres ${ELAPSED}s" >&2
    cat "$BODY_FILE" >&2
    echo >&2
    exit 1
fi

echo "OK en ${ELAPSED}s"
cat "$BODY_FILE"
echo

# Une source indisponible ne fait pas echouer le rafraichissement : les donnees
# deja stockees restent en place et la lacune est signalee (README.md §2).
# Elle doit rester visible dans le journal, sans pour autant marquer l'unite en
# echec — le sort des dossiers, lui, a bien ete recalcule.
grep -q '"registry_anomaly":null' "$BODY_FILE" \
    || echo "ANOMALIE : referentiel des acteurs non rafraichi, rattachements sur la version precedente" >&2
grep -q '"scrutins_anomaly":null' "$BODY_FILE" \
    || echo "ANOMALIE : scrutins non rafraichis, les scrutins stockes restent en place" >&2
