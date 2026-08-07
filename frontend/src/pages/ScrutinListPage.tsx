import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import ScrutinList, { CoverageNote } from '../components/ScrutinList'
import {
  Button,
  ErrorPanel,
  Loading,
  PageHeader,
  SegmentedControl,
} from '../components/ui'
import type { ScrutinListResponse } from '../types/scrutins'

const PAGE_SIZE = 50

const outcomes = [
  { value: '', label: 'Tous' },
  { value: 'adopté', label: 'Adoptés' },
  { value: 'rejeté', label: 'Rejetés' },
]

const ballotTypes = [
  { value: '', label: 'Tous' },
  { value: 'SPO', label: 'Ordinaires' },
  { value: 'SPS', label: 'Solennels' },
  { value: 'MOC', label: 'Motions de censure' },
]

const dossierFilters = [
  { value: '', label: 'Tous' },
  { value: 'true', label: 'Avec dossier' },
  { value: 'false', label: 'Sans dossier' },
]

export default function ScrutinListPage() {
  const [outcome, setOutcome] = useState('')
  const [ballotType, setBallotType] = useState('')
  const [withDossier, setWithDossier] = useState('')
  const [search, setSearch] = useState('')
  const [draft, setDraft] = useState('')
  const [offset, setOffset] = useState(0)

  const params = new URLSearchParams({
    limit: String(PAGE_SIZE),
    offset: String(offset),
  })
  if (outcome) params.set('outcome', outcome)
  if (ballotType) params.set('ballot_type', ballotType)
  if (withDossier) params.set('with_dossier', withDossier)
  if (search) params.set('search', search)

  const { data, isLoading, isError, error } = useQuery<ScrutinListResponse>({
    queryKey: ['scrutins', params.toString()],
    queryFn: () =>
      fetch(`/api/scrutins?${params}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  function reset(setter: (v: string) => void) {
    return (v: string) => {
      setter(v)
      setOffset(0)
    }
  }

  return (
    <>
      <PageHeader
        title="Scrutins"
        lede="Tous les scrutins publics de la législature, dans l'ordre où ils ont eu lieu. Aucune sélection, aucun classement."
      />

      {/* Recherche et filtres sur une seule ligne : la largeur du site les
          porte, et trois blocs empilés repoussaient la liste vers le bas. */}
      <div className="mb-4 flex flex-wrap items-center gap-x-4 gap-y-2">
        <form
          onSubmit={(e) => {
            e.preventDefault()
            setSearch(draft)
            setOffset(0)
          }}
          className="flex w-full gap-1.5 sm:w-auto"
        >
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Rechercher dans l'objet du scrutin…"
            className="min-w-0 flex-1 rounded-lg border border-line bg-surface px-3 py-1.5 text-sm shadow-card placeholder:text-ink-faint focus:border-accent focus:ring-2 focus:ring-accent/15 focus:outline-none sm:w-72 sm:flex-none"
          />
          <Button type="submit">Rechercher</Button>
        </form>

        <SegmentedControl
          label="Sort"
          options={outcomes}
          value={outcome}
          onChange={reset(setOutcome)}
        />
        <SegmentedControl
          label="Type"
          options={ballotTypes}
          value={ballotType}
          onChange={reset(setBallotType)}
        />
        <SegmentedControl
          label="Dossier"
          options={dossierFilters}
          value={withDossier}
          onChange={reset(setWithDossier)}
        />
      </div>

      {data && (
        <div className="mb-4">
          <CoverageNote note={data.coverage_note} />
        </div>
      )}

      {isLoading && <Loading>Chargement des scrutins…</Loading>}
      {isError && <ErrorPanel error={error} />}

      {data && (
        <>
          <p className="mb-2 text-xs text-ink-faint">
            {data.total.toLocaleString('fr-FR')} scrutin
            {data.total > 1 ? 's' : ''}
            {data.total > 0 &&
              ` — affichés ${data.offset + 1} à ${data.offset + data.count}`}
          </p>

          {data.count === 0 ? (
            <p className="rounded-xl border border-line bg-surface px-4 py-3 text-sm text-ink-soft shadow-card">
              Aucun scrutin ne correspond à ces filtres.
            </p>
          ) : (
            <ScrutinList scrutins={data.scrutins} />
          )}

          <div className="mt-5 flex items-center justify-between">
            <Button
              onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
              disabled={offset === 0}
            >
              ← Précédents
            </Button>
            <Button
              onClick={() => setOffset(offset + PAGE_SIZE)}
              disabled={offset + data.count >= data.total}
            >
              Suivants →
            </Button>
          </div>
        </>
      )}
    </>
  )
}
