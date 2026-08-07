import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router'
import OutcomeBadge from '../components/OutcomeBadge'
import {
  Button,
  ErrorPanel,
  ListCard,
  Loading,
  Meta,
  PageHeader,
  Pill,
} from '../components/ui'
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
      <PageHeader
        title="Dossiers législatifs"
        lede={
          data
            ? `Les ${data.total.toLocaleString('fr-FR')} dossiers ouverts sous cette législature, du plus récemment actif au plus ancien. Rien n'est écarté.`
            : 'Tous les dossiers ouverts sous cette législature, du plus récemment actif au plus ancien.'
        }
      />

      {isLoading && <Loading>Chargement des dossiers…</Loading>}
      {isError && <ErrorPanel error={error} />}

      {/*
        Une ligne par dossier plutôt qu'une carte : la largeur disponible porte
        le titre et la date sur la même ligne, et vingt dossiers tiennent dans
        une hauteur d'écran au lieu de trois.
      */}
      {data && (
        <ListCard muted={isPlaceholderData}>
          {data.dossiers.map((d) => (
            <Link
              key={d.uid}
              to={`/dossiers/${d.uid}`}
              className="group flex flex-col gap-1.5 px-4 py-3 transition-colors hover:bg-surface-soft sm:flex-row sm:items-baseline sm:justify-between sm:gap-6"
            >
              <div className="min-w-0 flex-1">
                <p className="text-[15px] font-semibold leading-snug text-ink transition-colors group-hover:text-accent">
                  {d.title}
                </p>
                <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1">
                  <Meta>{d.procedure}</Meta>
                  {d.current_stage && (
                    <Pill tone="info">
                      {d.current_stage.label}
                      {d.current_stage.chamber && ` — ${d.current_stage.chamber}`}
                    </Pill>
                  )}
                  <OutcomeBadge outcome={d.outcome} />
                </div>
              </div>
              <div className="shrink-0 sm:w-40 sm:text-right">
                <span className="block text-xs font-medium text-ink-soft">
                  {new Date(
                    d.last_activity_date + 'T00:00:00',
                  ).toLocaleDateString('fr-FR', {
                    day: 'numeric',
                    month: 'short',
                    year: 'numeric',
                  })}
                </span>
                <span className="block text-xs text-ink-faint">
                  {d.last_activity_label}
                </span>
              </div>
            </Link>
          ))}
        </ListCard>
      )}

      {data && data.total_pages > 1 && (
        <nav className="mt-5 flex items-center justify-between gap-4">
          <Button onClick={() => goTo(page - 1)} disabled={page <= 1}>
            ← Précédent
          </Button>
          <span className="text-xs text-ink-faint">
            Page {data.page} sur {data.total_pages}
          </span>
          <Button
            onClick={() => goTo(page + 1)}
            disabled={page >= data.total_pages}
          >
            Suivant →
          </Button>
        </nav>
      )}
    </>
  )
}
