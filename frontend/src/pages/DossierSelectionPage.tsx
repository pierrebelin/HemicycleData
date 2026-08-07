import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router'
import { AdminTokenField } from '../components/AdminTokenField'
import { adminFetch } from '../lib/adminToken'

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
  if (score >= 60) return 'bg-yes'
  if (score >= 30) return 'bg-abstain'
  return 'bg-ink-faint'
}

/// Écran de travail éditorial, hérité de l'ancienne cible « posts Instagram ».
/// Il reste hors du menu principal : le site public, lui, expose tous les
/// dossiers sans classement (README.md §2, §6).
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
    mutationFn: async ({ uid, status }: { uid: string; status: string }) => {
      await adminFetch(`/api/dossiers/${uid}/curate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['suggestions'] })
      queryClient.invalidateQueries({ queryKey: ['dossiers-recent'] })
    },
  })

  const refresh = useMutation({
    mutationFn: async () => {
      const response = await adminFetch('/api/refresh', { method: 'POST' })
      return response.json() as Promise<{ count: number }>
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dossiers'] })
      queryClient.invalidateQueries({ queryKey: ['dossiers-recent'] })
      queryClient.invalidateQueries({ queryKey: ['dossier'] })
      queryClient.invalidateQueries({ queryKey: ['suggestions'] })
    },
  })

  return (
    <>
      <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1 mb-3">
        <h2 className="text-2xl font-semibold tracking-tight">
          Sélection des dossiers
        </h2>
        <p className="text-sm text-ink-faint">
          Outil éditorial interne. Le score et le classement n'apparaissent pas
          sur les pages publiques.
        </p>
        <button
          onClick={() => refresh.mutate()}
          disabled={refresh.isPending}
          className="ml-auto flex items-center gap-2 rounded border border-line bg-surface px-3 py-1 text-sm text-ink-soft hover:bg-surface-soft disabled:opacity-50 disabled:cursor-not-allowed"
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

      {/* Curation et rafraîchissement écrivent : sans le jeton du jour, l'API
          répond 401 (src/api/security.rs). */}
      <div className="mb-3 max-w-md">
        <AdminTokenField />
      </div>

      {curate.isError && (
        <div className="mb-3 rounded-lg border border-no/25 bg-no-soft px-3 py-2">
          <p className="text-no text-sm">
            {curate.error instanceof Error ? curate.error.message : 'Erreur inconnue'}
          </p>
        </div>
      )}

      {refresh.isSuccess && (
        <div className="mb-3 rounded-lg border border-yes/25 bg-yes-soft px-3 py-2">
          <p className="text-yes text-sm">
            {refresh.data.count} dossier{refresh.data.count > 1 ? 's' : ''} synchronisé{refresh.data.count > 1 ? 's' : ''}
          </p>
        </div>
      )}

      {refresh.isError && (
        <div className="mb-3 rounded-lg border border-no/25 bg-no-soft px-3 py-2">
          <p className="text-no text-sm">
            Erreur de synchronisation : {refresh.error instanceof Error ? refresh.error.message : 'inconnue'}
          </p>
        </div>
      )}

      <section className="mb-5">
        <h3 className="mb-2 flex items-center gap-2 text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint">
          <span className="w-2 h-2 rounded-full bg-abstain-bar" />
          Suggestions Instagram
        </h3>
        {suggestions.data && suggestions.data.suggestions.length === 0 && (
          <p className="text-sm text-ink-faint">
            Aucun dossier en attente de sélection.
          </p>
        )}
        <div className="space-y-1.5">
          {suggestions.data?.suggestions.map((d) => (
            <div
              key={d.uid}
              className="flex items-center gap-4 rounded-lg border border-abstain/30 bg-abstain-soft px-4 py-2.5"
            >
              <Link
                to={`/dossiers/${d.uid}`}
                className="flex-1 min-w-0 hover:opacity-70"
              >
                <div className="flex items-center gap-2">
                  <span
                    className={`inline-flex items-center justify-center rounded px-1.5 text-xs font-bold text-white ${scoreBadgeColor(d.score_total)}`}
                  >
                    {d.score_total}
                  </span>
                  <p className="truncate text-sm font-medium leading-snug">
                    {d.title}
                  </p>
                </div>
                <p className="mt-0.5 text-xs text-ink-faint">{d.procedure}</p>
              </Link>
              <div className="flex gap-1.5 shrink-0">
                <button
                  onClick={() =>
                    curate.mutate({ uid: d.uid, status: 'selected' })
                  }
                  disabled={curate.isPending}
                  className="rounded border border-yes/25 bg-yes-soft px-2 py-0.5 text-xs font-medium text-yes hover:border-yes/50 disabled:opacity-50"
                >
                  Sélectionner
                </button>
                <button
                  onClick={() =>
                    curate.mutate({ uid: d.uid, status: 'dismissed' })
                  }
                  disabled={curate.isPending}
                  className="rounded border border-line bg-surface px-2 py-0.5 text-xs font-medium text-ink-soft hover:border-line-strong disabled:opacity-50"
                >
                  Écarter
                </button>
              </div>
            </div>
          ))}
        </div>
      </section>

      <section>
        <div className="mb-2 flex flex-wrap items-center gap-3">
          <h3 className="text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint">
            Dossiers actifs
          </h3>
          <div className="flex gap-0.5">
            {[7, 14, 30].map((d) => (
              <button
                key={d}
                onClick={() => setDays(d)}
                className={`rounded px-2 py-0.5 text-xs transition-colors ${
                  days === d
                    ? 'bg-accent-soft text-accent font-medium'
                    : 'text-ink-soft hover:bg-surface-soft'
                }`}
              >
                {d}j
              </button>
            ))}
          </div>
          {recent.data && (
            <span className="text-xs text-ink-faint">
              {recent.data.count} dossier{recent.data.count > 1 ? 's' : ''}
            </span>
          )}
        </div>

        {recent.isLoading && (
          <p className="animate-pulse py-8 text-ink-faint">
            Chargement des dossiers…
          </p>
        )}

        {recent.isError && (
          <div className="rounded-lg border border-no/25 bg-no-soft px-4 py-3">
            <p className="text-no text-sm">
              Erreur :{' '}
              {recent.error instanceof Error ? recent.error.message : 'inconnue'}
            </p>
          </div>
        )}

        {recent.data && (
          <div className="divide-y divide-line rounded-lg border border-line bg-surface">
            {recent.data.dossiers.map((d) => (
              <Link
                key={d.uid}
                to={`/dossiers/${d.uid}`}
                className="flex items-baseline justify-between gap-6 px-4 py-2.5 hover:bg-surface-soft transition-colors"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span
                      className={`inline-flex items-center justify-center rounded px-1.5 text-xs font-bold text-white ${scoreBadgeColor(d.score_total)}`}
                    >
                      {d.score_total}
                    </span>
                    <p className="truncate text-sm font-medium leading-snug">
                      {d.title}
                    </p>
                  </div>
                  <div className="mt-0.5 flex flex-wrap items-center gap-2 text-xs text-ink-faint">
                    <span>{d.procedure}</span>
                    {d.current_stage && (
                      <span className="rounded border border-info/20 bg-info-soft px-1.5 text-info">
                        {d.current_stage.label}
                        {d.current_stage.chamber && ` — ${d.current_stage.chamber}`}
                      </span>
                    )}
                  </div>
                </div>
                <div className="shrink-0 text-right">
                  <span className="block text-xs text-ink-soft">
                    {new Date(
                      d.last_activity_date + 'T00:00:00',
                    ).toLocaleDateString('fr-FR', {
                      day: 'numeric',
                      month: 'short',
                    })}
                  </span>
                  <span className="block text-xs text-ink-faint">
                    {d.last_activity_label}
                  </span>
                </div>
              </Link>
            ))}
          </div>
        )}
      </section>
    </>
  )
}
