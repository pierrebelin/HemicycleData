import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router'

interface StageDto {
  label: string
  chamber: string
}

interface DossierDto {
  uid: string
  title: string
  procedure: string
  last_activity_date: string
  last_activity_label: string
  score_total: number
  current_stage: StageDto | null
  curation_status: string
}

interface SuggestionsResponse {
  count: number
  suggestions: DossierDto[]
}

interface RecentDossiersResponse {
  count: number
  dossiers: DossierDto[]
}

function scoreBadgeColor(score: number) {
  if (score >= 60) return 'bg-emerald-600'
  if (score >= 30) return 'bg-amber-600'
  return 'bg-gray-600'
}

/// Écran de travail éditorial, hérité de l'ancienne cible « posts Instagram ».
/// Il reste hors du menu principal : le site public, lui, expose tous les
/// dossiers sans classement (PROJECT.md §2, §6).
export default function DossierSelectionPage() {
  const [days, setDays] = useState(7)
  const queryClient = useQueryClient()

  const recent = useQuery<RecentDossiersResponse>({
    queryKey: ['dossiers-recent', days],
    queryFn: () =>
      fetch(`/api/dossiers/recent?days=${days}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  const suggestions = useQuery<SuggestionsResponse>({
    queryKey: ['suggestions'],
    queryFn: () =>
      fetch('/api/suggestions?count=3').then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  const curate = useMutation({
    mutationFn: ({ uid, status }: { uid: string; status: string }) =>
      fetch(`/api/dossiers/${uid}/curate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      }).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['suggestions'] })
      queryClient.invalidateQueries({ queryKey: ['dossiers-recent'] })
    },
  })

  const refresh = useMutation({
    mutationFn: () =>
      fetch('/api/refresh', { method: 'POST' }).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json() as Promise<{ count: number }>
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dossiers'] })
      queryClient.invalidateQueries({ queryKey: ['dossiers-recent'] })
      queryClient.invalidateQueries({ queryKey: ['dossier'] })
      queryClient.invalidateQueries({ queryKey: ['suggestions'] })
    },
  })

  return (
    <>
      <div className="flex items-center gap-4 mb-2">
        <h2 className="text-xl font-semibold">Sélection des dossiers</h2>
        <button
          onClick={() => refresh.mutate()}
          disabled={refresh.isPending}
          className="ml-auto px-3 py-1 rounded text-sm bg-sunken text-ink-3 hover:bg-sunken-strong disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {refresh.isPending ? (
            <>
              <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
              </svg>
              Synchronisation…
            </>
          ) : (
            'Rafraîchir'
          )}
        </button>
      </div>
      <p className="text-sm text-ink-4 mb-8">
        Outil éditorial interne. Le score et le classement n'apparaissent pas
        sur les pages publiques.
      </p>

      {refresh.isSuccess && (
        <div className="mb-4 bg-for-softer border border-for-line rounded-lg p-3">
          <p className="text-for-ink text-sm">
            {refresh.data.count} dossier{refresh.data.count > 1 ? 's' : ''} synchronisé{refresh.data.count > 1 ? 's' : ''}
          </p>
        </div>
      )}

      {refresh.isError && (
        <div className="mb-4 bg-against-softer border border-against-line rounded-lg p-3">
          <p className="text-against-ink text-sm">
            Erreur de synchronisation : {refresh.error instanceof Error ? refresh.error.message : 'inconnue'}
          </p>
        </div>
      )}

      <section className="mb-8">
        <h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
          <span className="w-2 h-2 rounded-full bg-amber-500" />
          Suggestions Instagram
        </h3>
        {suggestions.data && suggestions.data.suggestions.length === 0 && (
          <p className="text-sm text-ink-4">
            Aucun dossier en attente de sélection.
          </p>
        )}
        <div className="space-y-2">
          {suggestions.data?.suggestions.map((d) => (
            <div
              key={d.uid}
              className="bg-raised border border-abstain-line rounded-lg p-4 flex items-center gap-4"
            >
              <Link
                to={`/dossiers/${d.uid}`}
                className="flex-1 min-w-0 hover:opacity-80"
              >
                <div className="flex items-center gap-2">
                  <span
                    className={`inline-flex items-center justify-center px-2 py-0.5 rounded text-xs font-bold text-white ${scoreBadgeColor(d.score_total)}`}
                  >
                    {d.score_total}
                  </span>
                  <p className="text-ink font-medium leading-snug truncate">
                    {d.title}
                  </p>
                </div>
                <p className="text-xs text-ink-4 mt-1">{d.procedure}</p>
              </Link>
              <div className="flex gap-2 shrink-0">
                <button
                  onClick={() =>
                    curate.mutate({ uid: d.uid, status: 'selected' })
                  }
                  disabled={curate.isPending}
                  className="px-3 py-1.5 rounded text-xs font-medium bg-for-soft border border-for-line text-for-ink hover:bg-for-soft-strong disabled:opacity-50"
                >
                  Sélectionner
                </button>
                <button
                  onClick={() =>
                    curate.mutate({ uid: d.uid, status: 'dismissed' })
                  }
                  disabled={curate.isPending}
                  className="px-3 py-1.5 rounded text-xs font-medium bg-sunken border border-line-strong text-ink-3 hover:bg-sunken-strong disabled:opacity-50"
                >
                  Écarter
                </button>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section>
        <div className="flex items-center gap-4 mb-3">
          <h3 className="text-lg font-semibold">Dossiers actifs</h3>
          <div className="flex gap-1">
            {[7, 14, 30].map((d) => (
              <button
                key={d}
                onClick={() => setDays(d)}
                className={`px-3 py-1 rounded text-sm ${
                  days === d
                    ? 'bg-blue-600 text-white'
                    : 'bg-sunken text-ink-3 hover:bg-sunken-strong'
                }`}
              >
                {d}j
              </button>
            ))}
          </div>
          {recent.data && (
            <span className="text-sm text-ink-4">
              {recent.data.count} dossier{recent.data.count > 1 ? 's' : ''}
            </span>
          )}
        </div>

        {recent.isLoading && (
          <p className="text-ink-3 animate-pulse py-8">
            Chargement des dossiers…
          </p>
        )}

        {recent.isError && (
          <div className="bg-against-softer border border-against-line rounded-lg p-4">
            <p className="text-against-ink">
              Erreur :{' '}
              {recent.error instanceof Error ? recent.error.message : 'inconnue'}
            </p>
          </div>
        )}

        <div className="space-y-3">
          {recent.data?.dossiers.map((d) => (
            <Link
              key={d.uid}
              to={`/dossiers/${d.uid}`}
              className="block bg-raised border border-line rounded-lg p-4 hover:border-line-stronger transition-colors"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span
                      className={`inline-flex items-center justify-center px-2 py-0.5 rounded text-xs font-bold text-white ${scoreBadgeColor(d.score_total)}`}
                    >
                      {d.score_total}
                    </span>
                    <p className="text-ink font-medium leading-snug truncate">
                      {d.title}
                    </p>
                  </div>
                  <div className="flex items-center gap-2 mt-1">
                    <p className="text-xs text-ink-4">{d.procedure}</p>
                    {d.current_stage && (
                      <span className="text-xs px-1.5 py-0.5 rounded bg-model-soft border border-model-line text-model-ink">
                        {d.current_stage.label}
                        {d.current_stage.chamber && ` — ${d.current_stage.chamber}`}
                      </span>
                    )}
                  </div>
                </div>
                <div className="flex flex-col items-end shrink-0 text-right">
                  <span className="text-xs text-ink-3">
                    {new Date(
                      d.last_activity_date + 'T00:00:00',
                    ).toLocaleDateString('fr-FR', {
                      day: 'numeric',
                      month: 'short',
                    })}
                  </span>
                  <span className="text-xs text-link-ink mt-0.5">
                    {d.last_activity_label}
                  </span>
                </div>
              </div>
            </Link>
          ))}
        </div>
      </section>
    </>
  )
}
