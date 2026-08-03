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

  if (text.isLoading) return <p className="text-gray-500">Chargement…</p>
  if (text.error)
    return <p className="text-red-400">{(text.error as Error).message}</p>

  const data = text.data!

  return (
    <div className="space-y-6">
      <div>
        <Link to="/themes" className="text-sm text-gray-500 hover:text-gray-300">
          ← Tous les thèmes
        </Link>
        <h2 className="mt-2 text-xl font-bold">{data.label}</h2>
        <p className="mt-1 text-xs text-gray-500">
          {data.scrutin_count} scrutin{data.scrutin_count > 1 ? 's' : ''}
          {data.first_vote && data.last_vote
            ? ` · du ${frenchDate(data.first_vote)} au ${frenchDate(data.last_vote)}`
            : ''}
        </p>
        {data.dossier_uid && (
          <p className="mt-1 text-xs text-gray-500">
            Dossier :{' '}
            <Link
              to={`/dossiers/${data.dossier_uid}`}
              className="underline hover:text-gray-300"
            >
              {data.dossier_label ?? data.dossier_uid}
            </Link>
          </p>
        )}
      </div>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">Familles</h3>
        {data.families.length > 0 ? (
          <FamilyBadges families={data.families} />
        ) : (
          <UnassignedNote outcome={data.last_attempt_outcome} />
        )}
        {data.families.some((f) => f.motive) && (
          <ul className="mt-3 space-y-1 text-xs text-gray-400">
            {data.families
              .filter((f) => f.motive)
              .map((f) => (
                <li key={f.code}>
                  <span className="text-gray-300">{f.label}</span> — {f.motive}
                </li>
              ))}
          </ul>
        )}
      </section>

      {data.proposal && (
        <section>
          <h3 className="mb-2 text-sm font-semibold text-gray-300">
            Proposition automatique
          </h3>
          <p className="text-xs text-gray-500">
            Modèle {data.proposal.model}, instruction{' '}
            {data.proposal.prompt_version}, le{' '}
            {frenchDate(data.proposal.produced_on)}. Le modèle n'a reçu que le
            libellé du texte et n'a produit aucun chiffre.
          </p>
          <ul className="mt-2 space-y-1 text-xs text-gray-400">
            {data.proposal.families.map((f) => (
              <li key={f.code}>
                <span className="text-gray-300">{f.label}</span> —{' '}
                {f.justification}
              </li>
            ))}
          </ul>
        </section>
      )}

      {data.history.length > 0 && (
        <section>
          <h3 className="mb-2 text-sm font-semibold text-gray-300">
            Historique des rattachements
          </h3>
          <ul className="space-y-1 text-xs text-gray-500">
            {data.history.map((entry, index) => (
              <li key={`${entry.code}-${entry.opened_on}-${index}`}>
                {entry.label} · {entry.origin_note} · du{' '}
                {frenchDate(entry.opened_on)}
                {entry.closed_on
                  ? ` au ${frenchDate(entry.closed_on)}`
                  : ' à aujourd’hui'}{' '}
                · {entry.author}
              </li>
            ))}
          </ul>
        </section>
      )}

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">
          Scrutins de ce texte
        </h3>
        {data.scrutins.length === 0 ? (
          <p className="text-sm text-gray-500">Aucun scrutin listé.</p>
        ) : (
          <ul className="space-y-2">
            {data.scrutins.map((scrutin) => (
              <li
                key={scrutin.uid}
                className="rounded border border-gray-800 bg-gray-900/40 p-3"
              >
                <Link
                  to={`/scrutins/${scrutin.uid}`}
                  className="text-sm hover:underline"
                >
                  {scrutin.subject}
                </Link>
                <p className="mt-1 text-xs text-gray-500">
                  {frenchDate(scrutin.date)} · scrutin n° {scrutin.number} ·{' '}
                  {scrutin.outcome_label} · pour {scrutin.votes_for}, contre{' '}
                  {scrutin.votes_against}, abstention {scrutin.abstentions}
                </p>
              </li>
            ))}
          </ul>
        )}
        {data.scrutin_count > data.scrutins.length && (
          <p className="mt-2 text-xs text-gray-500">
            {data.scrutins.length} scrutins affichés sur {data.scrutin_count}.
          </p>
        )}
      </section>
    </div>
  )
}
