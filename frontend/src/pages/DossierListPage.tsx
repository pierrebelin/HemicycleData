import { useState } from 'react'
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
  SegmentedControl,
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

/*
  Les cinq sorts que le domaine distingue, tous proposés : n'en offrir que
  trois laisserait croire que les deux autres n'existent pas. « Sans
  conclusion » est le libellé de la source, pas un jugement sur le dossier
  (README.md §6).
*/
const outcomes = [
  { value: '', label: 'Tous' },
  { value: 'promulgated', label: 'Promulgués' },
  { value: 'rejected', label: 'Rejetés' },
  { value: 'withdrawn', label: 'Retirés' },
  { value: 'merged_into', label: 'Absorbés' },
  { value: 'no_recorded_conclusion', label: 'Sans conclusion' },
]

const initiatives = [
  { value: '', label: 'Toutes' },
  { value: 'government', label: 'Projets de loi' },
  { value: 'parliamentary', label: 'Propositions' },
]

type Criterion = 'search' | 'outcome' | 'initiative' | 'page'

export default function DossierListPage() {
  const [params, setParams] = useSearchParams()
  const page = Math.max(1, Number(params.get('page')) || 1)
  const search = params.get('search') ?? ''
  const outcome = params.get('outcome') ?? ''
  const initiative = params.get('initiative') ?? ''

  const [draft, setDraft] = useState(search)

  const filtered = Boolean(search || outcome || initiative)

  const query = new URLSearchParams({
    page: String(page),
    per_page: String(PER_PAGE),
  })
  if (search) query.set('search', search)
  if (outcome) query.set('outcome', outcome)
  if (initiative) query.set('initiative', initiative)

  const { data, isLoading, isError, error, isPlaceholderData } =
    useQuery<DossierPageResponse>({
      queryKey: ['dossiers', query.toString()],
      queryFn: () =>
        fetch(`/api/dossiers?${query}`).then((res) => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`)
          return res.json()
        }),
      // Garde la page précédente affichée pendant le chargement de la suivante :
      // la liste ne clignote pas à chaque changement de page.
      placeholderData: keepPreviousData,
    })

  /*
    Les filtres vivent dans l'URL, comme la pagination : une liste filtrée se
    partage et se retrouve avec le bouton « précédent » du navigateur.
  */
  const apply = (changes: Partial<Record<Criterion, string>>) => {
    const next = new URLSearchParams(params)
    for (const [key, value] of Object.entries(changes)) {
      if (value) next.set(key, value)
      else next.delete(key)
    }
    // Un critère qui change ramène à la première page : la page 7 d'une liste
    // filtrée n'a rien à voir avec la page 7 de la précédente.
    if (!('page' in changes)) next.delete('page')
    setParams(next)
    window.scrollTo({ top: 0 })
  }

  const goTo = (next: number) => apply({ page: next === 1 ? '' : String(next) })

  const reset = () => {
    setDraft('')
    setParams({})
    window.scrollTo({ top: 0 })
  }

  return (
    <>
      <PageHeader
        title="Dossiers législatifs"
        lede={
          !data
            ? 'Tous les dossiers ouverts sous cette législature, du plus récemment actif au plus ancien.'
            : filtered
              ? /*
                  Sous filtre, le total affiché n'est plus celui de la base :
                  le dire évite de laisser croire que la législature se réduit
                  à ces dossiers (README.md §2).
                */
                `${data.total.toLocaleString('fr-FR')} dossier${data.total > 1 ? 's' : ''} correspond${data.total > 1 ? 'ent' : ''} à ces filtres. Les filtres restreignent l'affichage, jamais la base.`
              : `Les ${data.total.toLocaleString('fr-FR')} dossiers ouverts sous cette législature, du plus récemment actif au plus ancien. Rien n'est écarté.`
        }
        /*
          Les scrutins sont le détail d'un dossier avant d'être une rubrique :
          leur porte d'entrée est ici, à côté du titre, plutôt qu'en septième
          onglet de la barre de navigation.
        */
        aside={
          <Link
            to="/scrutins"
            className="text-sm font-medium text-accent hover:underline"
          >
            Accéder à tous les scrutins →
          </Link>
        }
      />

      {/* Recherche et filtres sur une seule ligne, comme sur les scrutins :
          la largeur du site les porte, et trois blocs empilés repousseraient
          la liste vers le bas. */}
      <div className="mb-4 flex flex-wrap items-center gap-x-4 gap-y-2">
        <form
          onSubmit={(e) => {
            e.preventDefault()
            apply({ search: draft.trim() })
          }}
          className="flex w-full gap-1.5 sm:w-auto"
        >
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Rechercher dans le titre du dossier…"
            className="min-w-0 flex-1 rounded-lg border border-line bg-surface px-3 py-1.5 text-sm shadow-card placeholder:text-ink-faint focus:border-accent focus:ring-2 focus:ring-accent/15 focus:outline-none sm:w-72 sm:flex-none"
          />
          <Button type="submit">Rechercher</Button>
        </form>

        <SegmentedControl
          label="Sort"
          options={outcomes}
          value={outcome}
          onChange={(value) => apply({ outcome: value })}
        />
        <SegmentedControl
          label="Initiative"
          options={initiatives}
          value={initiative}
          onChange={(value) => apply({ initiative: value })}
        />

        {filtered && (
          <button
            type="button"
            onClick={reset}
            className="text-xs font-medium text-accent hover:underline"
          >
            Tout afficher
          </button>
        )}
      </div>

      {isLoading && <Loading>Chargement des dossiers…</Loading>}
      {isError && <ErrorPanel error={error} />}

      {data && data.dossiers.length === 0 && (
        <p className="rounded-xl border border-line bg-surface px-4 py-3 text-sm text-ink-soft shadow-card">
          Aucun dossier ne correspond à ces filtres.
        </p>
      )}

      {/*
        Une ligne par dossier plutôt qu'une carte : la largeur disponible porte
        le titre et la date sur la même ligne, et vingt dossiers tiennent dans
        une hauteur d'écran au lieu de trois.
      */}
      {data && data.dossiers.length > 0 && (
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
