import { Link } from 'react-router'
import type { ScrutinSummaryDto } from '../types/scrutins'
import { formatDate } from '../types/scrutins'

function outcomeClasses(code: string) {
  return code === 'adopté'
    ? 'bg-for-soft border-for-line text-for-ink'
    : 'bg-against-soft border-against-line text-against-ink'
}

/** Chiffres bruts, jamais d'adverbe : « 280 contre 250 », pas « massivement ». */
export function TallyLine({
  tally,
}: {
  tally: ScrutinSummaryDto['tally']
}) {
  return (
    <span className="text-xs text-ink-3 tabular-nums">
      <span className="text-for-ink">{tally.votes_for}</span>
      <span className="text-ink-5"> pour · </span>
      <span className="text-against-ink">{tally.votes_against}</span>
      <span className="text-ink-5"> contre · </span>
      <span className="text-abstain-ink">{tally.abstentions}</span>
      <span className="text-ink-5"> abstention{tally.abstentions > 1 ? 's' : ''}</span>
      {tally.not_voting > 0 && (
        <span className="text-ink-4"> · {tally.not_voting} non-votant{tally.not_voting > 1 ? 's' : ''}</span>
      )}
      {tally.voluntary_not_voting > 0 && (
        <span className="text-ink-4">
          {' '}
          · {tally.voluntary_not_voting} non-votant{tally.voluntary_not_voting > 1 ? 's' : ''} volontaire
          {tally.voluntary_not_voting > 1 ? 's' : ''}
        </span>
      )}
    </span>
  )
}

/**
 * Lacune de couverture affichée telle quelle : les votes à main levée sont
 * absents de la source, le site ne peut rien en dire.
 */
export function CoverageNote({ note }: { note: string }) {
  return (
    <p className="text-xs text-ink-4 border border-line bg-raised rounded-md px-3 py-2">
      {note}
    </p>
  )
}

export default function ScrutinList({
  scrutins,
}: {
  scrutins: ScrutinSummaryDto[]
}) {
  return (
    <div className="space-y-2">
      {scrutins.map((s) => (
        <Link
          key={s.uid}
          to={`/scrutins/${s.uid}`}
          className="block bg-raised border border-line rounded-lg p-4 hover:border-line-stronger transition-colors"
        >
          <div className="flex items-start justify-between gap-4">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 flex-wrap">
                <span
                  className={`inline-flex items-center px-2 py-0.5 rounded border text-xs font-medium ${outcomeClasses(s.outcome_code)}`}
                >
                  {s.outcome_code}
                </span>
                <span className="text-xs text-ink-4">
                  n° {s.number} · {s.ballot_type}
                </span>
                {s.has_reconstructed_tallies && (
                  <span
                    className="text-xs px-1.5 py-0.5 rounded bg-info-soft border border-info-line text-info-ink"
                    title="La source ne publie pas les groupes sur ce scrutin : la répartition est reconstituée."
                  >
                    répartition reconstituée
                  </span>
                )}
              </div>
              <p className="text-ink text-sm leading-snug mt-1.5 line-clamp-2">
                {s.subject}
              </p>
              <div className="mt-1.5">
                <TallyLine tally={s.tally} />
              </div>
              {s.dossier_uid && (
                <p className="text-xs text-link-ink mt-1 truncate">
                  {s.dossier_label || s.dossier_uid}
                </p>
              )}
            </div>
            <span className="text-xs text-ink-3 shrink-0">
              {formatDate(s.date)}
            </span>
          </div>
        </Link>
      ))}
    </div>
  )
}
