#!/usr/bin/env bash
#
# Vérifie quotidiennement les sources déjà attribuées aux candidatures 2027.
# Une page modifiée est signalée dans le journal ; elle n'est jamais importée
# automatiquement : un nouvel extrait ou une nouvelle candidature exige une
# source primaire et une revue humaine (README.md §3.3 et §8.2).

set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL est requis}"

readonly CURL_TIMEOUT=45
readonly CURL_CONNECT_TIMEOUT=10
readonly TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

psql_base=(psql -X -q -v ON_ERROR_STOP=1 "$DATABASE_URL")

source_urls="$(${psql_base[@]} -At <<'SQL'
SELECT DISTINCT source_url
FROM (
    SELECT declaration_source_url AS source_url
    FROM presidential_candidates
    UNION
    SELECT official_site_url
    FROM presidential_candidates
    WHERE official_site_url IS NOT NULL
    UNION
    SELECT program_url
    FROM presidential_candidates
    WHERE program_url IS NOT NULL
    UNION
    SELECT source_url
    FROM candidate_political_organizations
    UNION
    SELECT source_url
    FROM candidate_program_proposals
) sources
ORDER BY source_url;
SQL
)"

if [[ -z "$source_urls" ]]; then
    echo "$(date -u +%FT%TZ) — aucune source de candidature a vérifier"
    exit 0
fi

record_observation() {
    local source_url="$1"
    local http_status="$2"
    local etag="$3"
    local last_modified="$4"
    local content_sha256="$5"

    "${psql_base[@]}" \
        --set=source_url="$source_url" \
        --set=http_status="$http_status" \
        --set=etag="$etag" \
        --set=last_modified="$last_modified" \
        --set=content_sha256="$content_sha256" <<'SQL' >/dev/null
INSERT INTO candidate_program_source_observations (
    source_url, http_status, etag, last_modified, content_sha256
) VALUES (
    :'source_url', :'http_status'::smallint, NULLIF(:'etag', ''),
    NULLIF(:'last_modified', ''), NULLIF(:'content_sha256', '')
)
ON CONFLICT (source_url) DO UPDATE SET
    last_checked_at = NOW(),
    http_status = EXCLUDED.http_status,
    etag = COALESCE(EXCLUDED.etag, candidate_program_source_observations.etag),
    last_modified = COALESCE(EXCLUDED.last_modified, candidate_program_source_observations.last_modified),
    content_sha256 = COALESCE(EXCLUDED.content_sha256, candidate_program_source_observations.content_sha256),
    last_changed_at = CASE
        WHEN EXCLUDED.content_sha256 IS NOT NULL
         AND candidate_program_source_observations.content_sha256 IS DISTINCT FROM EXCLUDED.content_sha256
        THEN NOW()
        ELSE candidate_program_source_observations.last_changed_at
    END;
SQL
}

failures=0

while IFS= read -r source_url; do
    state="$("${psql_base[@]}" -At -F $'\t' --set=source_url="$source_url" <<'SQL'
SELECT COALESCE(etag, ''), COALESCE(last_modified, ''), COALESCE(content_sha256, '')
FROM candidate_program_source_observations
WHERE source_url = :'source_url';
SQL
)"
    IFS=$'\t' read -r previous_etag previous_last_modified previous_hash <<< "$state"

    headers="$TEMP_DIR/headers"
    body="$TEMP_DIR/body"
    curl_args=(
        --location --silent --show-error --compressed
        --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_TIMEOUT"
        --dump-header "$headers" --output "$body" --write-out '%{http_code}'
    )
    [[ -n "$previous_etag" ]] && curl_args+=(--header "If-None-Match: $previous_etag")
    [[ -n "$previous_last_modified" ]] && curl_args+=(--header "If-Modified-Since: $previous_last_modified")

    if ! status="$(curl "${curl_args[@]}" "$source_url")"; then
        echo "$(date -u +%FT%TZ) — source inaccessible : $source_url" >&2
        record_observation "$source_url" 0 "" "" ""
        failures=$((failures + 1))
        continue
    fi

    etag="$(awk 'tolower($0) ~ /^etag:/ { value = $0; sub(/^[^:]*:[[:space:]]*/, "", value); sub(/\r$/, "", value); result = value } END { print result }' "$headers")"
    last_modified="$(awk 'tolower($0) ~ /^last-modified:/ { value = $0; sub(/^[^:]*:[[:space:]]*/, "", value); sub(/\r$/, "", value); result = value } END { print result }' "$headers")"

    if [[ "$status" == "304" ]]; then
        record_observation "$source_url" "$status" "$etag" "$last_modified" ""
        echo "$(date -u +%FT%TZ) — source inchangée : $source_url"
        continue
    fi

    if [[ ! "$status" =~ ^2[0-9][0-9]$ ]]; then
        echo "$(date -u +%FT%TZ) — source en erreur HTTP $status : $source_url" >&2
        record_observation "$source_url" "$status" "$etag" "$last_modified" ""
        failures=$((failures + 1))
        continue
    fi

    content_hash="$(sha256sum "$body" | awk '{print $1}')"
    if [[ -z "$previous_hash" ]]; then
        outcome="référencée pour la première fois"
    elif [[ "$previous_hash" == "$content_hash" ]]; then
        outcome="inchangée"
    else
        outcome="modifiée — revue humaine requise avant toute publication"
    fi

    record_observation "$source_url" "$status" "$etag" "$last_modified" "$content_hash"
    echo "$(date -u +%FT%TZ) — source $outcome : $source_url"
done <<< "$source_urls"

exit "$failures"
