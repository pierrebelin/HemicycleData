import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router'

interface DossierDto {
  uid: string
  title: string
  procedure: string
  last_activity_date: string
  last_activity_label: string
  score_total: number
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

export default function DossierListPage() {
  const [days, setDays] = useState(7)
  const queryClient = useQueryClient()

  const { data, isLoading, isError, error } =
    useQuery<RecentDossiersResponse>({
      queryKey: ['dossiers', days],
      queryFn: () =>
        fetch(`/api/dossiers?days=${days}`).then((res) => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`)
          return res.json()
        }),
    })

  const refresh = useMutation({
    mutationFn: () =>
      fetch('/api/refresh', { method: 'POST' }).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json() as Promise<{ count: number }>
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dossiers'] })
      queryClient.invalidateQueries({ queryKey: ['dossier'] })
    },
  })

  return (
    <>
      <div className="flex items-center gap-4 mb-8">
        <h2 className="text-xl font-semibold">Dossiers actifs</h2>
        <div className="flex gap-1">
          {[7, 14, 30].map((d) => (
            <button
              key={d}
              onClick={() => setDays(d)}
              className={`px-3 py-1 rounded text-sm ${
                days === d
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
              }`}
            >
              {d}j
            </button>
          ))}
        </div>
        {data && (
          <span className="text-sm text-gray-500">
            {data.count} dossier{data.count > 1 ? 's' : ''}
          </span>
        )}
        <button
          onClick={() => refresh.mutate()}
          disabled={refresh.isPending}
          className="ml-auto px-3 py-1 rounded text-sm bg-gray-800 text-gray-400 hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
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

      {refresh.isSuccess && (
        <div className="mb-4 bg-emerald-900/20 border border-emerald-800 rounded-lg p-3">
          <p className="text-emerald-400 text-sm">
            {refresh.data.count} dossier{refresh.data.count > 1 ? 's' : ''} synchronisé{refresh.data.count > 1 ? 's' : ''}
          </p>
        </div>
      )}

      {refresh.isError && (
        <div className="mb-4 bg-red-900/20 border border-red-800 rounded-lg p-3">
          <p className="text-red-400 text-sm">
            Erreur de synchronisation : {refresh.error instanceof Error ? refresh.error.message : 'inconnue'}
          </p>
        </div>
      )}

      {isLoading && (
        <div className="text-center py-20">
          <p className="text-gray-400 animate-pulse">
            Chargement des dossiers…
          </p>
        </div>
      )}

      {isError && (
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-4">
          <p className="text-red-400">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      )}

      <div className="space-y-3">
        {data?.dossiers.map((d) => (
          <Link
            key={d.uid}
            to={`/dossiers/${d.uid}`}
            className="block bg-gray-900 border border-gray-800 rounded-lg p-4 hover:border-gray-600 transition-colors"
          >
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span
                    className={`inline-flex items-center justify-center px-2 py-0.5 rounded text-xs font-bold text-white ${scoreBadgeColor(d.score_total)}`}
                  >
                    {d.score_total}
                  </span>
                  <p className="text-white font-medium leading-snug truncate">
                    {d.title}
                  </p>
                </div>
                <p className="text-xs text-gray-500 mt-1">{d.procedure}</p>
              </div>
              <div className="flex flex-col items-end shrink-0 text-right">
                <span className="text-xs text-gray-400">
                  {new Date(
                    d.last_activity_date + 'T00:00:00',
                  ).toLocaleDateString('fr-FR', {
                    day: 'numeric',
                    month: 'short',
                  })}
                </span>
                <span className="text-xs text-blue-400 mt-0.5">
                  {d.last_activity_label}
                </span>
              </div>
            </div>
          </Link>
        ))}
      </div>
    </>
  )
}
