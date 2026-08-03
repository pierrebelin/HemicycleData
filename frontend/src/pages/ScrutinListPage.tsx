import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import ScrutinList, { CoverageNote } from '../components/ScrutinList'
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

function FilterGroup({
  label,
  options,
  value,
  onChange,
}: {
  label: string
  options: { value: string; label: string }[]
  value: string
  onChange: (v: string) => void
}) {
  return (
    <div className="flex items-center gap-2">
      <span className="text-xs text-gray-500">{label}</span>
      <div className="flex gap-1">
        {options.map((o) => (
          <button
            key={o.value}
            onClick={() => onChange(o.value)}
            className={`px-2.5 py-1 rounded text-xs ${
              value === o.value
                ? 'bg-blue-600 text-white'
                : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
            }`}
          >
            {o.label}
          </button>
        ))}
      </div>
    </div>
  )
}

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
      <div className="mb-6">
        <h2 className="text-xl font-semibold mb-1">Scrutins</h2>
        <p className="text-sm text-gray-500">
          Tous les scrutins publics de la législature, sans sélection.
        </p>
      </div>

      <div className="space-y-3 mb-6">
        <form
          onSubmit={(e) => {
            e.preventDefault()
            setSearch(draft)
            setOffset(0)
          }}
          className="flex gap-2"
        >
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            placeholder="Rechercher dans l'objet du scrutin…"
            className="flex-1 bg-gray-900 border border-gray-800 rounded px-3 py-1.5 text-sm text-white placeholder-gray-600 focus:outline-none focus:border-gray-600"
          />
          <button
            type="submit"
            className="px-3 py-1.5 rounded text-sm bg-gray-800 text-gray-300 hover:bg-gray-700"
          >
            Rechercher
          </button>
        </form>

        <div className="flex flex-wrap gap-4">
          <FilterGroup
            label="Sort"
            options={outcomes}
            value={outcome}
            onChange={reset(setOutcome)}
          />
          <FilterGroup
            label="Type"
            options={ballotTypes}
            value={ballotType}
            onChange={reset(setBallotType)}
          />
          <FilterGroup
            label="Dossier"
            options={dossierFilters}
            value={withDossier}
            onChange={reset(setWithDossier)}
          />
        </div>

        {data && <CoverageNote note={data.coverage_note} />}
      </div>

      {isLoading && (
        <div className="text-center py-20">
          <p className="text-gray-400 animate-pulse">Chargement des scrutins…</p>
        </div>
      )}

      {isError && (
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-4">
          <p className="text-red-400">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      )}

      {data && (
        <>
          <p className="text-sm text-gray-500 mb-3 tabular-nums">
            {data.total.toLocaleString('fr-FR')} scrutin
            {data.total > 1 ? 's' : ''}
            {data.total > 0 && (
              <span className="text-gray-600">
                {' '}
                — affichés {data.offset + 1} à {data.offset + data.count}
              </span>
            )}
          </p>

          {data.count === 0 ? (
            <p className="text-gray-500 text-sm border border-gray-800 rounded-lg p-4">
              Aucun scrutin ne correspond à ces filtres.
            </p>
          ) : (
            <ScrutinList scrutins={data.scrutins} />
          )}

          <div className="flex items-center justify-between mt-6">
            <button
              onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
              disabled={offset === 0}
              className="px-3 py-1.5 rounded text-sm bg-gray-800 text-gray-300 hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              ← Précédents
            </button>
            <button
              onClick={() => setOffset(offset + PAGE_SIZE)}
              disabled={offset + data.count >= data.total}
              className="px-3 py-1.5 rounded text-sm bg-gray-800 text-gray-300 hover:bg-gray-700 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              Suivants →
            </button>
          </div>
        </>
      )}
    </>
  )
}
