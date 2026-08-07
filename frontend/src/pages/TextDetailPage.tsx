import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from 'react-router'
import { FamilyBadges, UnassignedNote } from '../components/ThemeBadges'
import type { TextDetailResponse } from '../types/themes'

function frenchDate(value: string) {
  return new Date(value).toLocaleDateString('fr-FR')
}

export default function TextDetailPage() {
  const { key } = useParams<{ key: string }>()

  const text = useQuery({
    queryKey: ['texts', key],
    queryFn: (): Promise<TextDetailResponse> =>
      fetch(`/api/texts/${encodeURIComponent(key!)}`).then((res) => {
        if (res.status === 404) throw new Error('Texte inconnu')
        if (!res.ok) throw new Error('Chargement impossible')
        return res.json()
      }),
  })

  if (text.isLoading) return <p className="text-ink-4">Chargement…</p>
  if (text.error)
    return <p className="text-against-ink">{(text.error as Error).message}</p>

  const data = text.data!

  return (
    <div className="space-y-6">
      <div>
        <Link to="/themes" className="text-sm text-ink-4 hover:text-ink-2">
          ← Tous les thèmes
        </Link>
        <h2 className="mt-2 text-xl font-bold">{data.label}</h2>
        <p className="mt-1 text-xs text-ink-4">
          {data.scrutin_count} scrutin{data.scrutin_count > 1 ? 's' : ''}
          {data.first_vote && data.last_vote
            ? ` · du ${frenchDate(data.first_vote)} au ${frenchDate(data.last_vote)}`
            : ''}
        </p>
        {data.dossier_uid && (
          <p className="mt-1 text-xs text-ink-4">
            Dossier :{' '}
            <Link
              to={`/dossiers/${data.dossier_uid}`}
              className="underline hover:text-ink-2"
            >
              {data.dossier_label ?? data.dossier_uid}
            </Link>
          </p>
        )}
      </div>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-ink-2">Familles</h3>
        {data.families.length > 0 ? (
          <FamilyBadges families={data.families} />
        ) : (
          <UnassignedNote outcome={data.last_attempt_outcome} />
        )}
        {data.families.some((f) => f.motive) && (
          <ul className="mt-3 space-y-1 text-xs text-ink-3">
            {data.families
              .filter((f) => f.motive)
              .map((f) => (
                <li key={f.code}>
                  <span className="text-ink-2">{f.label}</span> — {f.motive}
                </li>
              ))}
          </ul>
        )}
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-ink-2">
          Scrutins de ce texte
        </h3>
        {data.scrutins.length === 0 ? (
          <p className="text-sm text-ink-4">Aucun scrutin listé.</p>
        ) : (
          <ul className="space-y-2">
            {data.scrutins.map((scrutin) => (
              <li
                key={scrutin.uid}
                className="rounded border border-line bg-raised p-3"
              >
                <Link
                  to={`/scrutins/${scrutin.uid}`}
                  className="text-sm hover:underline"
                >
                  {scrutin.subject}
                </Link>
                <p className="mt-1 text-xs text-ink-4">
                  {frenchDate(scrutin.date)} · scrutin n° {scrutin.number} ·{' '}
                  {scrutin.outcome_label} · pour {scrutin.votes_for}, contre{' '}
                  {scrutin.votes_against}, abstention {scrutin.abstentions}
                </p>
              </li>
            ))}
          </ul>
        )}
        {data.scrutin_count > data.scrutins.length && (
          <p className="mt-2 text-xs text-ink-4">
            {data.scrutins.length} scrutins affichés sur {data.scrutin_count}.
          </p>
        )}
      </section>
    </div>
  )
}
