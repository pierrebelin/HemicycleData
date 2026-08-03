import { useState } from 'react'
import { Link, useParams } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import { CoverageNote, TallyLine } from '../components/ScrutinList'
import type { GroupBreakdownDto, ScrutinDetailDto } from '../types/scrutins'
import { formatDate, positionClasses, positionLabels } from '../types/scrutins'

function outcomeClasses(code: string) {
  return code === 'adopté'
    ? 'bg-emerald-900/30 border-emerald-800 text-emerald-300'
    : 'bg-red-900/30 border-red-800 text-red-300'
}

function GroupBlock({ group }: { group: GroupBreakdownDto }) {
  const [open, setOpen] = useState(false)
  const named = group.label ?? group.abbrev

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-lg overflow-hidden">
      <button
        onClick={() => setOpen(!open)}
        className="w-full text-left p-4 hover:bg-gray-900/60"
      >
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              {group.color && (
                <span
                  className="w-2.5 h-2.5 rounded-full shrink-0"
                  style={{ backgroundColor: group.color }}
                />
              )}
              {/* Libellé officiel du groupe, jamais traduit en parti. */}
              <span className="text-white text-sm font-medium">
                {named ??
                  (group.group_uid
                    ? group.group_uid
                    : 'Positions sans groupe publié')}
              </span>
              {group.abbrev && group.label && (
                <span className="text-xs text-gray-500">{group.abbrev}</span>
              )}
              {group.member_count !== null && (
                <span className="text-xs text-gray-600 tabular-nums">
                  {group.member_count} membres
                </span>
              )}
              {group.origin === 'reconstructed' && (
                <span className="text-xs px-1.5 py-0.5 rounded bg-sky-900/30 border border-sky-800 text-sky-300">
                  reconstituée
                </span>
              )}
            </div>
            <div className="mt-1.5">
              <TallyLine tally={group.tally} />
            </div>
            {group.group_uid === null && (
              <p className="text-xs text-gray-500 mt-1.5">
                La source ne rattache ces positions à aucun groupe. Elles sont
                affichées à part plutôt que réparties d'office.
              </p>
            )}
          </div>
          <span className="text-xs text-gray-500 shrink-0">
            {open
              ? 'Replier'
              : group.votes.length > 1
                ? `Voir les ${group.votes.length} votants`
                : `Voir le votant`}
          </span>
        </div>
      </button>

      {open && (
        <div className="border-t border-gray-800 divide-y divide-gray-800/60">
          {group.votes.length === 0 ? (
            <p className="p-4 text-xs text-gray-500">
              Aucune position nominale publiée pour cette ligne.
            </p>
          ) : (
            group.votes.map((v) => (
              <div
                key={v.actor_uid}
                className="flex items-center justify-between gap-3 px-4 py-2"
              >
                <span className="text-sm text-gray-200 truncate">
                  {v.official_url ? (
                    <a
                      href={v.official_url}
                      target="_blank"
                      rel="noreferrer"
                      className="hover:underline"
                    >
                      {v.full_name}
                    </a>
                  ) : (
                    /* Acteur absent du référentiel : identifiant brut, aucun nom deviné. */
                    v.full_name ?? (
                      <span className="text-gray-500 font-mono text-xs">
                        {v.actor_uid}
                      </span>
                    )
                  )}
                  {v.by_delegation && (
                    <span className="text-xs text-gray-500 ml-2">
                      par délégation
                    </span>
                  )}
                  {v.cause_code && (
                    <span className="text-xs text-gray-500 ml-2">
                      cause {v.cause_code}
                    </span>
                  )}
                </span>
                <span
                  className={`shrink-0 text-xs px-2 py-0.5 rounded border ${positionClasses[v.position]}`}
                >
                  {positionLabels[v.position]}
                </span>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  )
}

export default function ScrutinDetailPage() {
  const { uid } = useParams<{ uid: string }>()

  const { data, isLoading, isError, error } = useQuery<ScrutinDetailDto>({
    queryKey: ['scrutin', uid],
    queryFn: () =>
      fetch(`/api/scrutins/${uid}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
    enabled: !!uid,
  })

  if (isLoading) {
    return (
      <div className="text-center py-20">
        <p className="text-gray-400 animate-pulse">Chargement du scrutin…</p>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <Link to="/scrutins" className="text-blue-400 text-sm mb-4 inline-block">
          ← Retour aux scrutins
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

  const s = data.synthesis
  const methodNote = data.groups.find((g) => g.method_note)?.method_note

  return (
    <div>
      <Link to="/scrutins" className="text-blue-400 text-sm mb-6 inline-block">
        ← Retour aux scrutins
      </Link>

      <div className="mb-6">
        <div className="flex items-center gap-2 flex-wrap mb-2">
          <span
            className={`inline-flex items-center px-2 py-0.5 rounded border text-xs font-medium ${outcomeClasses(data.outcome_code)}`}
          >
            {data.outcome_label}
          </span>
          <span className="text-xs text-gray-500">
            Scrutin n° {data.number} · {data.ballot_type_label}
          </span>
          <span className="text-xs text-gray-500">{formatDate(data.date)}</span>
          {data.place && (
            <span className="text-xs text-gray-600">{data.place}</span>
          )}
        </div>
        <h2 className="text-xl font-semibold leading-snug">{data.subject}</h2>
        {data.majority_label && (
          <p className="text-xs text-gray-500 mt-2">{data.majority_label}</p>
        )}
        {data.requester && (
          <p className="text-xs text-gray-500 mt-1">
            Demandé par : {data.requester}
          </p>
        )}
        <div className="flex flex-wrap gap-3 mt-3 text-sm">
          <a
            href={data.official_url}
            target="_blank"
            rel="noreferrer"
            className="text-blue-400 hover:text-blue-300"
          >
            Page officielle du scrutin ↗
          </a>
          {data.dossier_uid && (
            <Link
              to={`/dossiers/${data.dossier_uid}`}
              className="text-blue-400 hover:text-blue-300"
            >
              {data.dossier_label || data.dossier_uid}
            </Link>
          )}
        </div>
      </div>

      <section className="bg-gray-900 border border-gray-800 rounded-lg p-5 mb-6">
        <h3 className="text-sm font-semibold text-gray-300 mb-3">
          Synthèse officielle
        </h3>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-3">
          {[
            ['Pour', s.tally.votes_for, 'text-emerald-400'],
            ['Contre', s.tally.votes_against, 'text-red-400'],
            ['Abstentions', s.tally.abstentions, 'text-amber-400'],
            ['Non-votants', s.tally.not_voting, 'text-gray-400'],
          ].map(([label, value, color]) => (
            <div key={label as string}>
              <p className={`text-2xl font-bold tabular-nums ${color}`}>
                {value as number}
              </p>
              <p className="text-xs text-gray-500">{label as string}</p>
            </div>
          ))}
        </div>
        <p className="text-xs text-gray-500 tabular-nums">
          {s.voters} votants · {s.expressed} suffrages exprimés ·{' '}
          {s.required} requis
        </p>
        <p className="text-xs text-gray-500 mt-1">{s.announcement}</p>
      </section>

      <section className="mb-6">
        <h3 className="text-sm font-semibold text-gray-300 mb-3">
          Répartition par groupe
        </h3>
        {/* Mention de méthode portée une fois pour toute la section : les
            chiffres marqués « reconstituée » sont produits par le site, pas
            publiés par l'Assemblée. */}
        {methodNote && (
          <p className="text-xs text-sky-300/80 border border-sky-900/60 bg-sky-950/20 rounded-md px-3 py-2 mb-3">
            {methodNote}
          </p>
        )}
        <div className="space-y-2">
          {data.groups.map((g, i) => (
            <GroupBlock key={g.group_uid ?? `orphans-${i}`} group={g} />
          ))}
        </div>
      </section>

      {data.corrections.length > 0 && (
        <section className="mb-6">
          <h3 className="text-sm font-semibold text-gray-300 mb-2">
            Mises au point
          </h3>
          {/* Déclarations postérieures des députés. Elles ne modifient aucun
              décompte publié par l'Assemblée. */}
          <p className="text-xs text-gray-500 mb-3">
            Déclarations postérieures au scrutin. Elles ne modifient ni la
            synthèse ni la répartition ci-dessus.
          </p>
          <div className="bg-gray-900 border border-gray-800 rounded-lg divide-y divide-gray-800/60">
            {data.corrections.map((c) => (
              <div
                key={c.actor_uid}
                className="flex items-center justify-between gap-3 px-4 py-2"
              >
                <span className="text-sm text-gray-200 truncate">
                  {c.full_name ?? (
                    <span className="text-gray-500 font-mono text-xs">
                      {c.actor_uid}
                    </span>
                  )}
                  {c.malfunction && (
                    <span className="text-xs text-gray-500 ml-2">
                      dysfonctionnement signalé
                    </span>
                  )}
                </span>
                <span className="shrink-0 text-xs text-gray-400">
                  souhaitait voter{' '}
                  <span
                    className={`px-2 py-0.5 rounded border ${positionClasses[c.claimed_position]}`}
                  >
                    {positionLabels[c.claimed_position]}
                  </span>
                </span>
              </div>
            ))}
          </div>
        </section>
      )}

      {data.unknown_actors > 0 && (
        <p className="text-xs text-gray-500 mb-4">
          {data.unknown_actors} votant{data.unknown_actors > 1 ? 's' : ''} ne
          figure{data.unknown_actors > 1 ? 'nt' : ''} pas au référentiel des
          acteurs : leur identifiant est affiché tel quel, aucun nom n'est
          deviné.
        </p>
      )}

      <CoverageNote note={data.coverage_note} />
    </div>
  )
}
