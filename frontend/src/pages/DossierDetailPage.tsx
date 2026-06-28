import { useParams, Link } from 'react-router'
import { useQuery } from '@tanstack/react-query'

interface ActeDto {
  date: string
  libelle: string
}

interface ScoreDto {
  avancement: number
  ampleur: number
  total: number
}

interface DossierDetailDto {
  uid: string
  titre: string
  procedure: string
  derniere_activite_date: string
  derniere_activite_libelle: string
  actes: ActeDto[]
  score: ScoreDto
}

function ScoreBar({
  label,
  value,
  weight,
}: {
  label: string
  value: number
  weight: number
}) {
  const pct = (value / 10) * 100
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-gray-300">
          {label}{' '}
          <span className="text-gray-500 text-xs">×{weight}</span>
        </span>
        <span className="text-gray-400">{value}/10</span>
      </div>
      <div className="h-2 bg-gray-800 rounded-full overflow-hidden">
        <div
          className="h-full bg-blue-500 rounded-full transition-all"
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  )
}

function scoreTotalColor(score: number) {
  if (score >= 60) return 'text-emerald-400'
  if (score >= 30) return 'text-amber-400'
  return 'text-gray-400'
}

export default function DossierDetailPage() {
  const { uid } = useParams<{ uid: string }>()

  const { data, isLoading, isError, error } = useQuery<DossierDetailDto>({
    queryKey: ['dossier', uid],
    queryFn: () =>
      fetch(`/api/dossiers/${uid}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
    enabled: !!uid,
  })

  if (isLoading) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-400 animate-pulse">Chargement du dossier…</p>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <Link
          to="/"
          className="text-blue-400 hover:text-blue-300 text-sm mb-4 inline-block"
        >
          ← Retour à la liste
        </Link>
        <div className="bg-red-900/20 border border-red-800 rounded-lg p-4">
          <p className="text-red-400">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      </div>
    )
  }

  if (!data) return null

  return (
    <div>
      <Link
        to="/"
        className="text-blue-400 hover:text-blue-300 text-sm mb-6 inline-block"
      >
        ← Retour à la liste
      </Link>

      <div className="mb-8">
        <h2 className="text-2xl font-bold leading-snug mb-2">{data.titre}</h2>
        <div className="flex items-center gap-3 text-sm">
          <span className="text-gray-400">{data.procedure}</span>
          <span className="text-gray-600">·</span>
          <span className="text-blue-400">
            {data.derniere_activite_libelle}
          </span>
          <span className="text-gray-600">·</span>
          <span className="text-gray-400">
            {new Date(
              data.derniere_activite_date + 'T00:00:00',
            ).toLocaleDateString('fr-FR', {
              day: 'numeric',
              month: 'long',
              year: 'numeric',
            })}
          </span>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-2 mb-8">
        <section className="bg-gray-900 border border-gray-800 rounded-lg p-5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold">Score</h3>
            <span
              className={`text-3xl font-bold ${scoreTotalColor(data.score.total)}`}
            >
              {data.score.total}
              <span className="text-base text-gray-500 font-normal">/100</span>
            </span>
          </div>
          <div className="space-y-3">
            <ScoreBar
              label="Avancement"
              value={data.score.avancement}
              weight={2}
            />
            <ScoreBar label="Ampleur" value={data.score.ampleur} weight={2} />
          </div>
          <p className="text-xs text-gray-600 mt-4">
            Critères objectifs uniquement. Les critères LLM (proximité
            thématique, impact concret, résonance actu) seront ajoutés
            ultérieurement.
          </p>
        </section>

        <section className="bg-gray-900 border border-gray-800 rounded-lg p-5">
          <h3 className="text-lg font-semibold mb-4">Timeline</h3>
          {data.actes.length === 0 ? (
            <p className="text-gray-500 text-sm">Aucun acte enregistré</p>
          ) : (
            <div className="relative pl-4 border-l border-gray-700 space-y-4">
              {data.actes.map((acte, i) => (
                <div key={i} className="relative">
                  <div className="absolute -left-[21px] top-1 w-2.5 h-2.5 rounded-full bg-blue-500 border-2 border-gray-900" />
                  <p className="text-sm text-white">{acte.libelle}</p>
                  <p className="text-xs text-gray-500">
                    {new Date(
                      acte.date + 'T00:00:00',
                    ).toLocaleDateString('fr-FR', {
                      day: 'numeric',
                      month: 'long',
                      year: 'numeric',
                    })}
                  </p>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>

      <div className="text-center">
        <a
          href={`https://www.assemblee-nationale.fr/dyn/17/dossiers/${data.uid}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-gray-500 hover:text-gray-300 underline"
        >
          Voir sur assemblee-nationale.fr ↗
        </a>
      </div>
    </div>
  )
}
