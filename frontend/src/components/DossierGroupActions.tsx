import { useQuery } from '@tanstack/react-query'
import { Card, ErrorPanel, Note, Pill, SectionTitle } from './ui'
import type {
  DossierGroupActionsResponse,
  GroupActionsDto,
  SummarySourceDto,
} from '../types/dossierGroupActions'

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
    <Card className="min-w-0 p-4">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div className="min-w-0">
          <h4 className="font-semibold text-ink">
            {group.abbrev} <span className="font-normal text-ink-soft">— {group.label}</span>
          </h4>
          {(group.start_date || group.end_date) && (
            <p className="mt-0.5 text-xs text-ink-faint">
              Groupe concerné du {group.start_date ? formatDate(group.start_date) : 'début de période'}
              {' '}au {group.end_date ? formatDate(group.end_date) : '—'}
            </p>
          )}
        </div>
        <a
          href={group.amendments_url}
          className="shrink-0 rounded-md border border-line px-2.5 py-1 text-xs text-ink-soft hover:bg-surface-soft hover:text-ink"
        >
          Amendements ({group.amendment_count})
        </a>
      </div>

      {group.summary && (
        <div className="mt-3 rounded-lg border border-line bg-surface-soft px-3 py-2.5">
          <div className="flex flex-wrap items-center gap-2">
            <Pill tone="info">{group.summary.label}</Pill>
          </div>
          <p className="mt-1.5 text-sm leading-relaxed text-ink-soft">{group.summary.text}</p>
          <SourceLinks sources={group.summary.sources} />
        </div>
      )}

      {!group.summary && group.state === 'summary_pending' && (
        <p className="mt-3 rounded-lg border border-line bg-surface-soft px-3 py-2 text-sm text-ink-faint">
          Synthèse automatique en attente de génération. Les actes ci-dessous restent consultables.
        </p>
      )}
      {!group.summary && group.state === 'summary_unavailable' && (
        <p className="mt-3 rounded-lg border border-line bg-surface-soft px-3 py-2 text-sm text-ink-faint">
          Synthèse automatique indisponible pour le moment. Les actes ci-dessous restent consultables.
        </p>
      )}
      {group.state === 'no_data' && (
        <p className="mt-3 rounded-lg border border-line bg-surface-soft px-3 py-2 text-sm text-ink-faint">
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
              <div key={vote.scrutin_uid} className="rounded-lg border border-line px-3 py-2">
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
    </Card>
  )
}

export default function DossierGroupActions({ uid }: { uid: string }) {
  const { data, isLoading, isError, error } = useQuery<DossierGroupActionsResponse>({
    queryKey: ['dossier-lecture-groupes', uid],
    queryFn: () =>
      fetch(`/api/dossiers/${uid}/lecture-groupes`).then((response) => {
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        return response.json()
      }),
    enabled: !!uid,
  })

  return (
    <section className="mb-6">
      <SectionTitle>Actes publiés par groupe</SectionTitle>
      <p className="mb-3 max-w-3xl text-sm leading-relaxed text-ink-soft">
        Une synthèse automatique descriptive peut accompagner les faits. Les votes finaux restent séparés par lecture et les exposés sommaires originaux restent dans les sections exhaustives ci-dessous.
      </p>
      {isLoading && <p className="animate-pulse text-sm text-ink-faint">Chargement des actes par groupe…</p>}
      {isError && <ErrorPanel error={error} />}
      {data && (
        <>
          <div className="grid min-w-0 gap-3 md:grid-cols-1">
            {data.groups.map((group) => <GroupCard key={group.uid} group={group} />)}
          </div>
          <Note>{data.notes.join(' ')}</Note>
        </>
      )}
    </section>
  )
}
