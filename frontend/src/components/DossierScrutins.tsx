import { useQuery } from '@tanstack/react-query'
import ScrutinList, { CoverageNote } from './ScrutinList'
import GuideLink from './GuideLink'
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
    <section className="mb-6">
      <h3 className="text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint mb-2">
        Scrutins
        {data && data.count > 0 && (
          <span className="ml-2 font-normal normal-case tracking-normal text-ink-faint">
            {data.count}
          </span>
        )}
      </h3>

      {isLoading && (
        <p className="text-ink-faint text-sm animate-pulse">
          Chargement des scrutins…
        </p>
      )}

      {isError && (
        <p className="text-no text-sm">
          Erreur : {error instanceof Error ? error.message : 'inconnue'}
        </p>
      )}

      {data && (
        <div className="space-y-2">
          {data.count === 0 ? (
            <p className="text-ink-soft text-sm border border-line bg-surface rounded-lg px-4 py-3">
              La source ne rattache aucun scrutin public à ce dossier. Cela ne
              signifie pas qu'aucun vote n'a eu lieu sur ce texte.{' '}
              <GuideLink ancre="q-sans-dossier">
                Pourquoi la plupart des scrutins n'ont pas de dossier
              </GuideLink>
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
