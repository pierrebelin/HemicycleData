import { useQuery } from '@tanstack/react-query'
import ScrutinList, { CoverageNote } from './ScrutinList'
import type { DossierScrutinsResponse } from '../types/scrutins'

/**
 * Section scrutins d'un dossier. Toujours présente, même vide : une section
 * absente laisserait croire qu'aucun vote n'a eu lieu, alors que la source peut
 * simplement ne rattacher aucun scrutin à ce dossier.
 */
export default function DossierScrutins({ uid }: { uid: string }) {
  const { data, isLoading, isError, error } = useQuery<DossierScrutinsResponse>({
    queryKey: ['dossier-scrutins', uid],
    queryFn: () =>
      fetch(`/api/dossiers/${uid}/scrutins`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
    enabled: !!uid,
  })

  return (
    <section className="mb-8">
      <h3 className="text-lg font-semibold mb-3">
        Scrutins
        {data && data.count > 0 && (
          <span className="text-sm text-gray-500 font-normal ml-2">
            {data.count}
          </span>
        )}
      </h3>

      {isLoading && (
        <p className="text-gray-500 text-sm animate-pulse">
          Chargement des scrutins…
        </p>
      )}

      {isError && (
        <p className="text-red-400 text-sm">
          Erreur : {error instanceof Error ? error.message : 'inconnue'}
        </p>
      )}

      {data && (
        <div className="space-y-3">
          {data.count === 0 ? (
            <p className="text-gray-500 text-sm border border-gray-800 rounded-lg p-4">
              La source ne rattache aucun scrutin public à ce dossier.
            </p>
          ) : (
            <ScrutinList scrutins={data.scrutins} />
          )}
          <CoverageNote note={data.coverage_note} />
        </div>
      )}
    </section>
  )
}
