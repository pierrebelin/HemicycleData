import { useState } from 'react'
import { Link, useParams } from 'react-router'
import { useQuery } from '@tanstack/react-query'
import GuideLink from '../components/GuideLink'
import Hemicycle from '../components/Hemicycle'
import { positions, siegesDesGroupes, tallyDesVotes } from '../components/positions'
import { CoverageNote, TallyLine } from '../components/ScrutinList'
import type { GroupBreakdownDto, ScrutinDetailDto } from '../types/scrutins'
import { formatDate, positionClasses, positionLabels } from '../types/scrutins'

function outcomeClasses(code: string) {
  return code === 'adopté'
    ? 'bg-for-soft border-for-line text-for-ink'
    : 'bg-against-soft border-against-line text-against-ink'
}

function GroupBlock({ group }: { group: GroupBreakdownDto }) {
  const [open, setOpen] = useState(false)
  const named = group.label ?? group.abbrev

  return (
    <div className="bg-raised border border-line rounded-lg overflow-hidden">
      <button
        onClick={() => setOpen(!open)}
        className="w-full text-left p-4 hover:bg-sunken"
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
              <span className="text-ink text-sm font-medium">
                {named ??
                  (group.group_uid
                    ? group.group_uid
                    : 'Positions sans groupe publié')}
              </span>
              {group.abbrev && group.label && (
                <span className="text-xs text-ink-4">{group.abbrev}</span>
              )}
              {group.member_count !== null && (
                <span className="text-xs text-ink-5 tabular-nums">
                  {group.member_count} membres
                </span>
              )}
              {group.origin === 'reconstructed' && (
                <span className="text-xs px-1.5 py-0.5 rounded bg-info-soft border border-info-line text-info-ink">
                  reconstituée
                </span>
              )}
            </div>
            <div className="mt-1.5">
              <TallyLine tally={group.tally} />
            </div>
            {group.group_uid === null && (
              <p className="text-xs text-ink-4 mt-1.5">
                La source ne rattache ces positions à aucun groupe. Elles sont
                affichées à part plutôt que réparties d'office.
              </p>
            )}
          </div>
          <span className="text-xs text-ink-4 shrink-0">
            {open
              ? 'Replier'
              : group.votes.length > 1
                ? `Voir les ${group.votes.length} votants`
                : `Voir le votant`}
          </span>
        </div>
      </button>

      {open && (
        <div className="border-t border-line divide-y divide-line">
          {group.votes.length === 0 ? (
            <p className="p-4 text-xs text-ink-4">
              Aucune position nominale publiée pour cette ligne.
            </p>
          ) : (
            group.votes.map((v) => (
              <div
                key={v.actor_uid}
                className="flex items-center justify-between gap-3 px-4 py-2"
              >
                <span className="text-sm text-ink-1 truncate">
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
                      <span className="text-ink-4 font-mono text-xs">
                        {v.actor_uid}
                      </span>
                    )
                  )}
                  {v.by_delegation && (
                    <span className="text-xs text-ink-4 ml-2">
                      par délégation
                    </span>
                  )}
                  {v.cause_code && (
                    <span className="text-xs text-ink-4 ml-2">
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
        <p className="text-ink-3 animate-pulse">Chargement du scrutin…</p>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <Link to="/scrutins" className="text-link-ink text-sm mb-4 inline-block">
          ← Retour aux scrutins
        </Link>
        <div className="bg-against-softer border border-against-line rounded-lg p-4">
          <p className="text-against-ink">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      </div>
    )
  }

  if (!data) return null

  const s = data.synthesis
  const methodNote = data.groups.find((g) => g.method_note)?.method_note
  const votesNominaux = siegesDesGroupes(data.groups)

  return (
    <div>
      <Link to="/scrutins" className="text-link-ink text-sm mb-6 inline-block">
        ← Retour aux scrutins
      </Link>

      <div className="mb-6">
        <div className="flex items-center gap-2 flex-wrap mb-2">
          <span
            className={`inline-flex items-center px-2 py-0.5 rounded border text-xs font-medium ${outcomeClasses(data.outcome_code)}`}
          >
            {data.outcome_label}
          </span>
          <span className="text-xs text-ink-4">
            Scrutin n° {data.number} · {data.ballot_type_label}
          </span>
          <span className="text-xs text-ink-4">{formatDate(data.date)}</span>
          {data.place && (
            <span className="text-xs text-ink-5">{data.place}</span>
          )}
        </div>
        <h2 className="text-xl font-semibold leading-snug">{data.subject}</h2>
        <p className="mt-2 text-xs text-ink-4">
          Le vote porte sur cet objet, pas nécessairement sur l'ensemble du
          texte.{' '}
          <GuideLink ancre="q-scrutin-loi">Comment lire un scrutin</GuideLink>
        </p>
        {data.majority_label && (
          <p className="text-xs text-ink-4 mt-2">{data.majority_label}</p>
        )}
        {data.requester && (
          <p className="text-xs text-ink-4 mt-1">
            Demandé par : {data.requester}
          </p>
        )}
        <div className="flex flex-wrap gap-3 mt-3 text-sm">
          <a
            href={data.official_url}
            target="_blank"
            rel="noreferrer"
            className="text-link-ink hover:text-link-ink"
          >
            Page officielle du scrutin ↗
          </a>
          {data.dossier_uid && (
            <Link
              to={`/dossiers/${data.dossier_uid}`}
              className="text-link-ink hover:text-link-ink"
            >
              {data.dossier_label || data.dossier_uid}
            </Link>
          )}
        </div>
        {!data.dossier_uid && (
          <p className="mt-2 text-xs text-ink-4">
            La source ne rattache ce scrutin à aucun dossier législatif. C'est
            le cas de la majorité des scrutins.{' '}
            <GuideLink ancre="q-sans-dossier">Pourquoi</GuideLink>
          </p>
        )}
      </div>

      <section className="bg-raised border border-line rounded-lg p-5 mb-6">
        <h3 className="text-sm font-semibold text-ink-2 mb-3">
          Synthèse officielle
        </h3>
        <div className="grid grid-cols-2 sm:grid-cols-4 gap-4 mb-3">
          {[
            ['Pour', s.tally.votes_for, 'text-for-ink'],
            ['Contre', s.tally.votes_against, 'text-against-ink'],
            ['Abstentions', s.tally.abstentions, 'text-abstain-ink'],
            ['Non-votants', s.tally.not_voting, 'text-ink-3'],
          ].map(([label, value, color]) => (
            <div key={label as string}>
              <p className={`text-2xl font-bold tabular-nums ${color}`}>
                {value as number}
              </p>
              <p className="text-xs text-ink-4">{label as string}</p>
            </div>
          ))}
        </div>
        <p className="text-xs text-ink-4 tabular-nums">
          {s.voters} votants · {s.expressed} suffrages exprimés ·{' '}
          {s.required} requis
        </p>
        <p className="text-xs text-ink-4 mt-1">{s.announcement}</p>
        <p className="mt-2 text-xs text-ink-4">
          Les abstentions n'entrent pas dans les suffrages exprimés, et
          « non-votant » ne se lit pas comme une absence.{' '}
          <GuideLink ancre="q-non-votant">
            Ce que recouvrent les positions
          </GuideLink>
        </p>
      </section>

      {votesNominaux.length > 0 && (
        <section className="bg-raised border border-line rounded-lg p-5 mb-6">
          <h3 id="hemicycle-scrutin" className="text-sm font-semibold text-ink-2">
            Chaque député à sa place
          </h3>
          <p className="mt-1 text-xs text-ink-4">
            Survolez un siège pour lire le nom, le groupe et la position.
          </p>

          <Hemicycle votes={votesNominaux} labelledBy="hemicycle-scrutin" />

          <div className="flex flex-wrap justify-center gap-x-5 gap-y-1.5 text-xs tabular-nums">
            {positions(tallyDesVotes(votesNominaux)).map((bloc) => (
              <span key={bloc.cle} className="flex items-center gap-1.5">
                <span
                  className={`h-2 w-2 rounded-full ${bloc.fond}`}
                  aria-hidden
                />
                <span className={bloc.texte}>{bloc.total}</span>
                <span className="text-ink-4">{bloc.libelleAccorde}</span>
              </span>
            ))}
          </div>

          <p className="mt-4 border-t border-line pt-3 text-xs leading-relaxed text-ink-4">
            Chaque député est à sa place réelle : la source publie le numéro de
            siège avec chaque position, et le{' '}
            <a
              href="https://www.assemblee-nationale.fr/dyn/vos-deputes/hemicycle"
              target="_blank"
              rel="noreferrer"
              className="underline decoration-line-strong underline-offset-2 hover:text-ink-2"
            >
              plan de l'hémicycle
            </a>{' '}
            donne l'emplacement de ce numéro. En gris, les sièges dont aucune
            position n'a été enregistrée sur ce scrutin. Le décompte ci-dessus
            est celui des positions nominales — la synthèse officielle compte
            les non-votants volontaires à part.
          </p>
        </section>
      )}

      <section className="mb-6">
        <h3 className="text-sm font-semibold text-ink-2 mb-3">
          Répartition par groupe
        </h3>
        <p className="mb-3 text-xs text-ink-4">
          Groupe du député à la date du scrutin. Un groupe n'est pas un parti.{' '}
          <GuideLink ancre="q-groupe">Pourquoi cette distinction</GuideLink>
        </p>
        {/* Mention de méthode portée une fois pour toute la section : les
            chiffres marqués « reconstituée » sont produits par le site, pas
            publiés par l'Assemblée. */}
        {methodNote && (
          <p className="text-xs text-info-ink border border-info-line bg-info-soft rounded-md px-3 py-2 mb-3">
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
          <h3 className="text-sm font-semibold text-ink-2 mb-2">
            Mises au point
          </h3>
          {/* Déclarations postérieures des députés. Elles ne modifient aucun
              décompte publié par l'Assemblée. */}
          <p className="text-xs text-ink-4 mb-3">
            Déclarations postérieures au scrutin. Elles ne modifient ni la
            synthèse ni la répartition ci-dessus.
          </p>
          <div className="bg-raised border border-line rounded-lg divide-y divide-line">
            {data.corrections.map((c) => (
              <div
                key={c.actor_uid}
                className="flex items-center justify-between gap-3 px-4 py-2"
              >
                <span className="text-sm text-ink-1 truncate">
                  {c.full_name ?? (
                    <span className="text-ink-4 font-mono text-xs">
                      {c.actor_uid}
                    </span>
                  )}
                  {c.malfunction && (
                    <span className="text-xs text-ink-4 ml-2">
                      dysfonctionnement signalé
                    </span>
                  )}
                </span>
                <span className="shrink-0 text-xs text-ink-3">
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
        <p className="text-xs text-ink-4 mb-4">
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
