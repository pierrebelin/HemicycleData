import { useQuery } from '@tanstack/react-query'
import { useSearchParams } from 'react-router'
import { GroupDot } from '../pages/GroupListPage'
import { Card, ErrorPanel, Note, Pill, SectionTitle, TallyLine, VoteBar } from './ui'
import type {
  DossierGroupActionsResponse,
  GroupActionsDto,
  SummarySourceDto,
} from '../types/dossierGroupActions'

const MAX_COMPARED_GROUPS = 4

function formatDate(iso: string) {
  return new Date(iso + 'T00:00:00').toLocaleDateString('fr-FR', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  })
}

function SourceLinks({ sources }: { sources: SummarySourceDto[] }) {
  return (
    <div className="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-xs">
      <span className="text-ink-faint">Sources :</span>
      {sources.map((source) =>
        source.official_url ? (
          <a
            key={source.id}
            href={source.official_url}
            target="_blank"
            rel="noreferrer"
            className="text-accent hover:underline"
          >
            {source.label} ↗
          </a>
        ) : (
          <span key={source.id} className="text-ink-soft" title={source.id}>
            {source.label}
          </span>
        ),
      )}
    </div>
  )
}

function Tally({ vote }: { vote: GroupActionsDto['final_votes'][number] }) {
  const { tally } = vote
  return (
    <span className="text-xs text-ink-faint">
      pour {tally.votes_for} · contre {tally.votes_against} · abstention{' '}
      {tally.abstentions} · non-votants {tally.not_voting + tally.voluntary_not_voting}
    </span>
  )
}

function GroupCard({ group }: { group: GroupActionsDto }) {
  return (
    <Card className="min-w-0 overflow-hidden">
      <details>
        <summary className="flex cursor-pointer list-none flex-wrap items-center justify-between gap-x-3 gap-y-1 px-3 py-2.5 hover:bg-surface-soft [&::-webkit-details-marker]:hidden">
          <span className="min-w-0 text-sm font-semibold text-ink">
            {group.abbrev} <span className="font-normal text-ink-soft">— {group.label}</span>
          </span>
          <span className="shrink-0 text-xs text-ink-faint">
            {group.final_votes.length} vote{group.final_votes.length > 1 ? 's' : ''} final{group.final_votes.length > 1 ? 'aux' : ''} · {group.amendment_count} amendement{group.amendment_count > 1 ? 's' : ''} · Détails
          </span>
        </summary>
        <div className="border-t border-line px-3 py-3">
          {(group.start_date || group.end_date) && (
            <p className="text-xs text-ink-faint">
              Groupe concerné du {group.start_date ? formatDate(group.start_date) : 'début de période'} au {group.end_date ? formatDate(group.end_date) : '—'}
            </p>
          )}

      {group.summary && (
        <div className="mt-2 rounded-lg border border-line bg-surface-soft px-3 py-2">
          <div className="flex flex-wrap items-center gap-2">
            <Pill tone="info">{group.summary.label}</Pill>
          </div>
          <p className="mt-1 text-sm leading-relaxed text-ink-soft">{group.summary.text}</p>
          <SourceLinks sources={group.summary.sources} />
        </div>
      )}

      {!group.summary && group.state === 'summary_pending' && (
        <p className="mt-2 text-xs text-ink-faint">
          Synthèse automatique en attente de génération. Les actes ci-dessous restent consultables.
        </p>
      )}
      {!group.summary && group.state === 'summary_unavailable' && (
        <p className="mt-2 text-xs text-ink-faint">
          Synthèse automatique indisponible pour le moment. Les actes ci-dessous restent consultables.
        </p>
      )}
      {group.state === 'no_data' && (
        <p className="mt-2 text-xs text-ink-faint">
          Aucun vote final ni amendement associé n'est disponible pour ce groupe dans ce dossier.
          Cette absence ne vaut pas abstention.
        </p>
      )}

      {group.final_votes.length > 0 && (
        <div className="mt-3">
          <p className="mb-1.5 text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint">
            Votes finaux, par lecture
          </p>
          <div className="space-y-1.5">
            {group.final_votes.map((vote) => (
              <div key={vote.scrutin_uid} className="rounded-lg border border-line px-2.5 py-2">
                <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1 text-sm">
                  <a href={`/scrutins/${vote.scrutin_uid}`} className="font-medium text-accent hover:underline">
                    Scrutin n° {vote.number}
                  </a>
                  <span className="text-xs text-ink-faint">{formatDate(vote.date)}</span>
                  {vote.reading && <span className="text-xs text-ink-soft">· {vote.reading}</span>}
                  <Pill tone={vote.outcome_code === 'adopted' ? 'yes' : vote.outcome_code === 'rejected' ? 'no' : 'neutral'}>
                    {vote.outcome_label}
                  </Pill>
                </div>
                <p className="mt-1 text-xs leading-relaxed text-ink-soft">{vote.text_label}</p>
                <div className="mt-1 flex flex-wrap gap-x-3 gap-y-1">
                  <Tally vote={vote} />
                  {vote.majority_position && (
                    <span className="text-xs text-ink-soft">
                      Position majoritaire publiée : {vote.majority_position}
                    </span>
                  )}
                  <a href={vote.official_url} target="_blank" rel="noreferrer" className="text-xs text-accent hover:underline">
                    Source officielle ↗
                  </a>
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
          <a
            href={group.amendments_url.replace('?', '?tab=tableaux&')}
            className="mt-3 inline-block rounded-md border border-line px-2.5 py-1 text-xs text-ink-soft hover:bg-surface-soft hover:text-ink"
          >
            Amendements ({group.amendment_count})
          </a>
        </div>
      </details>
    </Card>
  )
}

function comparisonGridCols(count: number) {
  if (count >= 4) return 'sm:grid-cols-2 xl:grid-cols-4'
  if (count === 3) return 'sm:grid-cols-3'
  if (count === 2) return 'sm:grid-cols-2'
  return ''
}

function GroupPicker({
  groups,
  selected,
  onToggle,
}: {
  groups: GroupActionsDto[]
  selected: string[]
  onToggle: (uid: string) => void
}) {
  const full = selected.length >= MAX_COMPARED_GROUPS

  return (
    <div className="mb-3 flex flex-wrap items-center gap-x-3 gap-y-2">
      <span className="text-xs font-medium text-ink-faint">
        Groupes comparés {selected.length}/{MAX_COMPARED_GROUPS}
      </span>
      <div className="flex flex-wrap gap-1.5">
        {groups.map((group) => {
          const isSelected = selected.includes(group.uid)
          return (
            <button
              key={group.uid}
              type="button"
              onClick={() => onToggle(group.uid)}
              aria-pressed={isSelected}
              disabled={full && !isSelected}
              title={group.label}
              className={`flex items-center gap-1.5 rounded-lg px-2.5 py-1 text-[13px] font-medium ring-1 ring-inset transition-colors ${
                isSelected
                  ? 'bg-ink text-white ring-ink'
                  : 'bg-surface text-ink-soft shadow-card ring-line hover:text-ink hover:ring-line-strong disabled:cursor-not-allowed disabled:opacity-40'
              }`}
            >
              <GroupDot color={group.color} />
              {group.abbrev}
            </button>
          )
        })}
      </div>
    </div>
  )
}

/** Fiche alignée d'un groupe : faits publiés, sans score ni conclusion. */
function ComparedGroup({ group }: { group: GroupActionsDto }) {
  return (
    <Card className="min-w-0 p-3">
      <div className="flex items-baseline gap-2">
        <GroupDot color={group.color} />
        <h4 className="text-sm font-semibold text-ink">{group.abbrev}</h4>
        <span className="truncate text-xs text-ink-faint" title={group.label}>{group.label}</span>
      </div>

      {group.summary ? (
        <div className="mt-3 border-t border-line pt-2.5">
          <Pill tone="info">{group.summary.label}</Pill>
          <p className="mt-1.5 text-sm leading-relaxed text-ink-soft">{group.summary.text}</p>
          <SourceLinks sources={group.summary.sources} />
        </div>
      ) : group.state === 'no_data' ? (
        <p className="mt-3 border-t border-line pt-2.5 text-xs leading-relaxed text-ink-faint">
          Aucun vote final ni amendement associé n'est disponible pour ce groupe. Cette absence ne vaut pas abstention.
        </p>
      ) : (
        <p className="mt-3 border-t border-line pt-2.5 text-xs text-ink-faint">
          Synthèse automatique indisponible ; les votes publiés restent ci-dessous.
        </p>
      )}

      <a
        href={group.amendments_url.replace('?', '?tab=tableaux&')}
        className="mt-3 inline-flex rounded-md border border-line px-2.5 py-1 text-xs font-medium text-ink-soft hover:bg-surface-soft hover:text-ink"
      >
        {group.amendment_count} amendement{group.amendment_count > 1 ? 's' : ''}
      </a>

      <div className="mt-3 space-y-2">
        <p className="text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint">Votes finaux, par lecture</p>
        {group.final_votes.length === 0 ? (
          <p className="text-xs text-ink-faint">Aucun vote final publié.</p>
        ) : (
          group.final_votes.map((vote) => (
            <div key={vote.scrutin_uid} className="rounded-lg border border-line bg-surface-soft px-2.5 py-2">
              <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                <a href={`/scrutins/${vote.scrutin_uid}`} className="text-xs font-medium text-accent hover:underline">
                  Scrutin n° {vote.number}
                </a>
                {vote.reading && <span className="text-xs text-ink-faint">· {vote.reading}</span>}
                <Pill tone={vote.outcome_code === 'adopted' ? 'yes' : vote.outcome_code === 'rejected' ? 'no' : 'neutral'}>
                  {vote.outcome_label}
                </Pill>
              </div>
              <div className="mt-2">
                <VoteBar tally={vote.tally} />
              </div>
              <div className="mt-1.5"><TallyLine tally={vote.tally} /></div>
            </div>
          ))
        )}
      </div>
    </Card>
  )
}

export default function DossierGroupActions({ uid }: { uid: string }) {
  const [params, setParams] = useSearchParams()
  const { data, isLoading, isError, error } = useQuery<DossierGroupActionsResponse>({
    queryKey: ['dossier-lecture-groupes', uid],
    queryFn: () =>
      fetch(`/api/dossiers/${uid}/lecture-groupes`).then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        return response.json()
      }),
    enabled: !!uid,
  })

  const selectedUids = (params.get('avis_groupes') ?? '')
    .split(',')
    .filter(Boolean)
    .slice(0, MAX_COMPARED_GROUPS)
  const selected = data?.groups.filter((group) => selectedUids.includes(group.uid)) ?? []

  function toggleGroup(groupUid: string) {
    const next = selectedUids.includes(groupUid)
      ? selectedUids.filter((uid) => uid !== groupUid)
      : [...selectedUids, groupUid].slice(0, MAX_COMPARED_GROUPS)
    const updated = new URLSearchParams(params)
    if (next.length === 0) updated.delete('avis_groupes')
    else updated.set('avis_groupes', next.join(','))
    setParams(updated)
  }

  return (
    <section className="mb-6">
      <SectionTitle>Actes publiés par groupe</SectionTitle>
      <p className="mb-2 max-w-3xl text-xs leading-relaxed text-ink-soft">
        Choisissez jusqu'à quatre groupes pour mettre côte à côte leurs synthèses et leurs votes finaux publiés. Aucun groupe n'est présélectionné.
      </p>
      {isLoading && <p className="animate-pulse text-sm text-ink-faint">Chargement des actes par groupe…</p>}
      {isError && <ErrorPanel error={error} />}
      {data && (
        <>
          <GroupPicker groups={data.groups} selected={selectedUids} onToggle={toggleGroup} />
          {selected.length > 0 ? (
            <div className={`grid min-w-0 gap-3 ${comparisonGridCols(selected.length)}`}>
              {selected.map((group) => <ComparedGroup key={group.uid} group={group} />)}
            </div>
          ) : (
            <div className="grid min-w-0 gap-2 md:grid-cols-1">
              {data.groups.map((group) => <GroupCard key={group.uid} group={group} />)}
            </div>
          )}
          <div className="mt-2">
            <Note>{data.notes.join(' ')}</Note>
          </div>
        </>
      )}
    </section>
  )
}
