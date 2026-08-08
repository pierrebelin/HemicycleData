#!/usr/bin/env bash
#
# Affiche le jeton d'administration du jour.
#
# Le jeton n'est pas stocke : il est derive du secret et de la date UTC, et il
# change a minuit. Le serveur accepte celui du jour et celui de la veille.
#
#   ~/app/deploy/bin/admin-token.sh
#   ssh hemicycle@<IP_DU_VPS> '~/app/deploy/bin/admin-token.sh'
#
# Le secret passe par l'environnement, jamais par la ligne de commande : `ps`
# expose `argv` a tous les utilisateurs de la machine.

set -euo pipefail

APP_DIR="${HEMICYCLE_APP_DIR:-$HOME/app}"
ENV_FILE="${HEMICYCLE_ENV_FILE:-$HOME/shared/.env}"
BINARY="$APP_DIR/target/release/admin-token"

if [[ -z "${ADMIN_TOKEN_SECRET:-}" ]]; then
    if [[ ! -r "$ENV_FILE" ]]; then
        echo "Secret introuvable : ni ADMIN_TOKEN_SECRET dans l'environnement, ni $ENV_FILE lisible." >&2
        exit 1
    fi
    set -a
    # shellcheck disable=SC1090
    . "$ENV_FILE"
    set +a
fi

if [[ ! -x "$BINARY" ]]; then
    echo "Binaire absent : $BINARY (lancer deploy.sh, ou cargo build --release)." >&2
    exit 1
fi

exec "$BINARY"
