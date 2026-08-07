import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router'
import OutcomeBadge from '../components/OutcomeBadge'
import type { OutcomeDto } from '../types/dossiers'

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
  current_stage: StageDto | null
  outcome: OutcomeDto
}

interface DossierPageResponse {
  page: number
  per_page: number
  total: number
  total_pages: number
  dossiers: DossierDto[]
}

const PER_PAGE = 20

export default function DossierListPage() {
  const [params, setParams] = useSearchParams()
  const page = Math.max(1, Number(params.get('page')) || 1)

  const { data, isLoading, isError, error, isPlaceholderData } =
    useQuery<DossierPageResponse>({
      queryKey: ['dossiers', page],
      queryFn: () =>
        fetch(`/api/dossiers?page=${page}&per_page=${PER_PAGE}`).then((res) => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`)
          return res.json()
        }),
      // Garde la page précédente affichée pendant le chargement de la suivante :
      // la liste ne clignote pas à chaque changement de page.
      placeholderData: keepPreviousData,
    })

  const goTo = (next: number) => {
    setParams(next === 1 ? {} : { page: String(next) })
    window.scrollTo({ top: 0 })
  }

  return (
    <>
      <div className="flex items-baseline gap-3 mb-8">
        <h2 className="text-xl font-semibold">Dossiers législatifs</h2>
        {data && (
          <span className="text-sm text-ink-4">
            {data.total.toLocaleString('fr-FR')} dossier
            {data.total > 1 ? 's' : ''}, du plus récent au plus ancien
          </span>
        )}
      </div>

      {isLoading && (
        <div className="text-center py-20">
          <p className="text-ink-3 animate-pulse">
            Chargement des dossiers…
          </p>
        </div>
      )}

      {isError && (
        <div className="bg-against-softer border border-against-line rounded-lg p-4">
          <p className="text-against-ink">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      )}

      <div
        className={`space-y-3 ${isPlaceholderData ? 'opacity-50' : ''}`}
      >
        {data?.dossiers.map((d) => (
          <Link
            key={d.uid}
            to={`/dossiers/${d.uid}`}
            className="block bg-raised border border-line rounded-lg p-4 hover:border-line-stronger transition-colors"
          >
            <div className="flex items-start justify-between gap-4">
              <div className="flex-1 min-w-0">
                <p className="text-ink font-medium leading-snug">{d.title}</p>
                <div className="flex items-center gap-2 mt-1">
                  <p className="text-xs text-ink-4">{d.procedure}</p>
                  {d.current_stage && (
                    <span className="text-xs px-1.5 py-0.5 rounded bg-model-soft border border-model-line text-model-ink">
                      {d.current_stage.label}
                      {d.current_stage.chamber && ` — ${d.current_stage.chamber}`}
                    </span>
                  )}
                  <OutcomeBadge outcome={d.outcome} />
                </div>
              </div>
              <div className="flex flex-col items-end shrink-0 text-right">
                <span className="text-xs text-ink-3">
                  {new Date(
                    d.last_activity_date + 'T00:00:00',
                  ).toLocaleDateString('fr-FR', {
                    day: 'numeric',
                    month: 'short',
                    year: 'numeric',
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

      {data && data.total_pages > 1 && (
        <nav className="flex items-center justify-between gap-4 mt-8">
          <button
            onClick={() => goTo(page - 1)}
            disabled={page <= 1}
            className="px-3 py-1.5 rounded text-sm bg-sunken text-ink-2 hover:bg-sunken-strong disabled:opacity-40 disabled:cursor-not-allowed"
          >
            ← Précédent
          </button>
          <span className="text-sm text-ink-4">
            Page {data.page} sur {data.total_pages}
          </span>
          <button
            onClick={() => goTo(page + 1)}
            disabled={page >= data.total_pages}
            className="px-3 py-1.5 rounded text-sm bg-sunken text-ink-2 hover:bg-sunken-strong disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Suivant →
          </button>
        </nav>
      )}
    </>
  )
}
