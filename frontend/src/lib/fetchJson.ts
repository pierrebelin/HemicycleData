const DEFAULT_TIMEOUT_MS = 8_000

/**
 * Une page publique ne doit pas rester indéfiniment sur « Chargement… » si
 * l'API est indisponible. Le délai borne l'attente côté lecteur sans modifier
 * les données ni masquer l'erreur : les composants affichent ensuite leur état
 * d'indisponibilité explicite.
 */
export async function fetchJson<T>(
  url: string,
  timeoutMs = DEFAULT_TIMEOUT_MS,
): Promise<T> {
  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), timeoutMs)

  try {
    const response = await fetch(url, { signal: controller.signal })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    return (await response.json()) as T
  } catch (error) {
    if (controller.signal.aborted) {
      throw new Error('Les données ne répondent pas pour le moment.')
    }
    throw error
  } finally {
    window.clearTimeout(timeout)
  }
}
