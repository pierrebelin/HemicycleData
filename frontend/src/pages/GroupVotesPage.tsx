import { useQuery } from '@tanstack/react-query'
import { Link, useSearchParams } from 'react-router'
import { FamilyBadges } from '../components/ThemeBadges'
import type {
  FinalVoteDto,
  FinalVoteListResponse,
  GroupDto,
  StanceDto,
} from '../types/finalVotes'
import type { FamiliesResponse } from '../types/themes'
import { formatDate } from '../types/scrutins'

const PAGE_SIZE = 20

/**
 * Couleurs des positions, jamais des groupes : le vert et le rouge disent
 * « pour » et « contre », pas « bien » et « mal ». La couleur du groupe reste
 * cantonnée à sa pastille d'identité.
 */
const POSITION_CLASSES: Record<string, string> = {
  for: 'text-emerald-300',
  against: 'text-red-300',
  abstention: 'text-amber-300',
}

function outcomeClasses(adopted: boolean) {
  return adopted
    ? 'bg-emerald-900/30 border-emerald-800 text-emerald-300'
    : 'bg-red-900/30 border-red-800 text-red-300'
}

/** Barre des trois parts. Elle rend visible ce que le pourcentage résume. */
function ShareBar({ share }: { share: NonNullable<StanceDto['share']> }) {
  return (
    <div className="flex h-1.5 w-full overflow-hidden rounded bg-gray-800">
      <div className="bg-emerald-500" style={{ width: `${share.for_percent}%` }} />
      <div className="bg-red-500" style={{ width: `${share.against_percent}%` }} />
      <div className="bg-amber-500" style={{ width: `${share.abstention_percent}%` }} />
    </div>
  )
}

/**
 * Position d'un groupe sur un vote. Le pourcentage ne remplace jamais les
 * chiffres bruts : les deux sont affichés côte à côte (PROJECT.md §6).
 */
function StanceCard({ stance }: { stance: StanceDto }) {
  const { share, tally } = stance
  // Le pourcentage porte la position réellement majoritaire chez les votants,
  // pas la position publiée : sur le scrutin 8431, la source annonce « contre »
  // pour un groupe dont 87 % des votants s'abstiennent.
  const leading = share?.leading_label ?? null
  const divergent =
    stance.majority !== null && share?.leading !== null && stance.majority !== share?.leading

  return (
    <div className="flex-1 min-w-0 rounded border border-gray-800 bg-gray-950/60 p-3">
      <div className="flex items-baseline gap-2">
        <span
          className="inline-block h-2.5 w-2.5 shrink-0 rounded-full border border-gray-700"
          style={{ backgroundColor: stance.color ?? 'transparent' }}
          aria-hidden
        />
        <span className="text-sm font-medium text-gray-200">{stance.abbrev}</span>
        {leading ? (
          <span
            className={`text-sm font-semibold ${POSITION_CLASSES[share?.leading ?? ''] ?? 'text-gray-300'}`}
          >
            {leading}
            {share?.leading_percent !== null && (
              <span className="tabular-nums"> {share?.leading_percent} %</span>
            )}
          </span>
        ) : (
          // Égalité stricte entre deux positions : aucune ne l'emporte, et en
          // désigner une au hasard serait une invention.
          share !== null && (
            <span className="text-sm text-gray-400">
              {share.tied_labels.join(' et ')} à égalité
            </span>
          )
        )}
      </div>

      {share === null ? (
        <p className="mt-2 text-xs text-gray-500">
          Aucun membre du groupe ne s'est prononcé sur ce vote.
        </p>
      ) : (
        <>
          <div className="mt-2">
            <ShareBar share={share} />
          </div>
          <p className="mt-1.5 text-xs tabular-nums text-gray-400">
            <span className="text-emerald-400">{tally.votes_for}</span> pour ·{' '}
            <span className="text-red-400">{tally.votes_against}</span> contre ·{' '}
            <span className="text-amber-400">{tally.abstentions}</span> abst.
            <span className="text-gray-600">
              {' '}
              — sur {share.voters} votant{share.voters > 1 ? 's' : ''}
            </span>
          </p>
        </>
      )}

      {divergent && stance.majority_label && (
        <p className="mt-1 text-xs text-gray-500">
          Position majoritaire publiée par l'Assemblée : {stance.majority_label}.
        </p>
      )}

      {(tally.not_voting > 0 || tally.voluntary_not_voting > 0) && (
        <p className="mt-0.5 text-xs tabular-nums text-gray-600">
          {tally.not_voting > 0 && `${tally.not_voting} non-votant${tally.not_voting > 1 ? 's' : ''}`}
          {tally.not_voting > 0 && tally.voluntary_not_voting > 0 && ' · '}
          {tally.voluntary_not_voting > 0 &&
            `${tally.voluntary_not_voting} non-votant${tally.voluntary_not_voting > 1 ? 's' : ''} volontaire${tally.voluntary_not_voting > 1 ? 's' : ''}`}
        </p>
      )}
    </div>
  )
}

/**
 * Un groupe sélectionné mais absent du scrutin : la lacune est nommée, jamais
 * comblée par un zéro (PROJECT.md §2).
 */
function MissingStance({ group }: { group: GroupDto }) {
  return (
    <div className="flex-1 min-w-0 rounded border border-dashed border-gray-800 p-3">
      <span className="text-sm font-medium text-gray-400">{group.abbrev}</span>
      <p className="mt-2 text-xs text-gray-600">
        Aucune ligne pour ce groupe sur ce scrutin.
      </p>
    </div>
  )
}

function VoteRow({ vote, selected }: { vote: FinalVoteDto; selected: GroupDto[] }) {
  return (
    <article className="rounded-lg border border-gray-800 bg-gray-900 p-4">
      <div className="flex flex-wrap items-center gap-2">
        <span
          className={`inline-flex items-center rounded border px-2 py-0.5 text-xs font-medium ${outcomeClasses(vote.adopted)}`}
        >
          {vote.adopted ? 'adopté' : 'rejeté'}
        </span>
        <span className="text-xs text-gray-500">
          {formatDate(vote.date)} · n° {vote.number} · {vote.ballot_type_label}
        </span>
        {vote.reading && (
          <span className="rounded border border-gray-700 px-1.5 py-0.5 text-xs text-gray-400">
            {vote.reading}
          </span>
        )}
      </div>

      <h3 className="mt-2 text-base leading-snug">
        <Link
          to={`/textes/${encodeURIComponent(vote.text_key)}`}
          className="text-gray-100 hover:underline"
        >
          {vote.text_label}
        </Link>
      </h3>

      <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1.5">
        <FamilyBadges families={vote.families} />
        {vote.dossier_uid && (
          <Link
            to={`/dossiers/${vote.dossier_uid}`}
            className="text-xs text-gray-500 underline hover:text-gray-300"
          >
            Dossier législatif
          </Link>
        )}
        <Link
          to={`/scrutins/${vote.scrutin_uid}`}
          className="text-xs text-gray-500 underline hover:text-gray-300"
        >
          Détail du scrutin
        </Link>
      </div>

      {selected.length > 0 && (
        <div className="mt-3 flex flex-col gap-2 sm:flex-row">
          {selected.map((group) => {
            const stance = vote.stances.find((s) => s.group_uid === group.uid)
            return stance ? (
              <StanceCard key={group.uid} stance={stance} />
            ) : (
              <MissingStance key={group.uid} group={group} />
            )
          })}
        </div>
      )}

      <p className="mt-2 text-xs tabular-nums text-gray-600">
        Assemblée entière : {vote.synthesis.votes_for} pour ·{' '}
        {vote.synthesis.votes_against} contre · {vote.synthesis.abstentions} abst.
      </p>
    </article>
  )
}

function GroupSelect({
  label,
  value,
  groups,
  allowEmpty,
  onChange,
}: {
  label: string
  value: string
  groups: GroupDto[]
  allowEmpty: boolean
  onChange: (value: string) => void
}) {
  return (
    <label className="flex items-center gap-2 text-xs text-gray-500">
      {label}
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded border border-gray-800 bg-gray-900 px-2 py-1 text-sm text-gray-200 focus:border-gray-600 focus:outline-none"
      >
        {allowEmpty && <option value="">Aucun</option>}
        {groups.map((group) => (
          <option key={group.uid} value={group.abbrev}>
            {group.abbrev} — {group.label}
          </option>
        ))}
      </select>
    </label>
  )
}

/**
 * Votes sur l'ensemble d'un texte, groupe par groupe.
 *
 * Le groupe parlementaire est l'unité affichée, jamais le parti : ce ne sont
 * pas les mêmes objets, et une équivalence approximative présentée comme un
 * fait serait une fausse information (PROJECT.md §3.1).
 */
export default function GroupVotesPage() {
  const [params, setParams] = useSearchParams()

  const groupsParam = params.get('groupes') ?? 'RN,SOC'
  const theme = params.get('theme') ?? ''
  const offset = Math.max(0, Number(params.get('offset') ?? 0) || 0)

  const query = new URLSearchParams({
    groupes: groupsParam,
    limit: String(PAGE_SIZE),
    offset: String(offset),
  })
  if (theme) query.set('theme', theme)

  const { data, isLoading, isError, error } = useQuery<FinalVoteListResponse>({
    queryKey: ['votes-finaux', query.toString()],
    queryFn: () =>
      fetch(`/api/votes-finaux?${query}`).then(async (res) => {
        if (!res.ok) throw new Error((await res.text()) || `HTTP ${res.status}`)
        return res.json()
      }),
  })

  const families = useQuery<FamiliesResponse>({
    queryKey: ['themes'],
    queryFn: () =>
      fetch('/api/themes').then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  const [groupA = '', groupB = ''] = groupsParam
    .split(',')
    .map((token) => token.trim())
    .filter(Boolean)

  function update(next: Record<string, string>) {
    const merged = new URLSearchParams(params)
    for (const [key, value] of Object.entries(next)) {
      if (value) merged.set(key, value)
      else merged.delete(key)
    }
    // Tout changement de filtre ramène en tête de liste : garder l'ancien
    // décalage afficherait une page vide sans le dire. La pagination, elle,
    // pose son propre décalage et n'est pas concernée.
    if (!('offset' in next)) merged.delete('offset')
    setParams(merged)
  }

  function setGroups(a: string, b: string) {
    update({ groupes: [a, b].filter(Boolean).join(',') })
  }

  const groups = data?.groups ?? []
  const selected = data?.selected ?? []
  const shown = data?.items.length ?? 0

  return (
    <>
      <div className="mb-6">
        <h2 className="mb-1 text-xl font-semibold">Votes par groupe</h2>
        <p className="text-sm text-gray-500">
          Le vote sur l'ensemble de chaque texte, groupe parlementaire par groupe
          parlementaire.
        </p>
      </div>

      <div className="mb-6 space-y-3">
        <div className="flex flex-wrap items-center gap-4">
          <GroupSelect
            label="Groupe"
            value={groupA}
            groups={groups}
            allowEmpty={false}
            onChange={(value) => setGroups(value, groupB)}
          />
          <GroupSelect
            label="Comparer à"
            value={groupB}
            groups={groups.filter((g) => g.abbrev !== groupA)}
            allowEmpty
            onChange={(value) => setGroups(groupA, value)}
          />
          <label className="flex items-center gap-2 text-xs text-gray-500">
            Thème
            <select
              value={theme}
              onChange={(e) => update({ theme: e.target.value })}
              className="rounded border border-gray-800 bg-gray-900 px-2 py-1 text-sm text-gray-200 focus:border-gray-600 focus:outline-none"
            >
              <option value="">Tous</option>
              {(families.data?.families ?? []).map((family) => (
                <option key={family.code} value={family.code}>
                  {family.label}
                </option>
              ))}
            </select>
          </label>
        </div>

        {data && (
          <div className="space-y-1.5">
            <p className="rounded-md border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-500">
              {data.scope_note}{' '}
              <Link to="/scrutins" className="underline hover:text-gray-300">
                Voir tous les scrutins
              </Link>
            </p>
            <p className="rounded-md border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-500">
              {data.share_note} {data.outcome_note}
            </p>
            {selected
              .filter((group) => group.final_vote_count < data.total_unfiltered)
              .map((group) => (
                <p
                  key={group.uid}
                  className="rounded-md border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-500"
                >
                  {group.label} ({group.abbrev}) n'apparaît que sur{' '}
                  {group.final_vote_count} des {data.total_unfiltered} votes sur
                  l'ensemble : le groupe n'existait pas sur les autres.
                </p>
              ))}
            {theme && (
              <p className="rounded-md border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-500">
                La thématisation est en cours : {data.total_with_family} des{' '}
                {data.total_unfiltered} votes sur l'ensemble portent une famille.
                Filtrer par thème laisse donc de côté les textes pas encore
                rattachés.{' '}
                <Link to="/themes/methode" className="underline hover:text-gray-300">
                  Méthode
                </Link>
              </p>
            )}
          </div>
        )}
      </div>

      {isLoading && (
        <div className="py-20 text-center">
          <p className="animate-pulse text-gray-400">Chargement des votes…</p>
        </div>
      )}

      {isError && (
        <div className="rounded-lg border border-red-800 bg-red-900/20 p-4">
          <p className="text-red-400">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      )}

      {data && (
        <>
          <p className="mb-3 text-sm tabular-nums text-gray-500">
            {data.total.toLocaleString('fr-FR')} vote{data.total > 1 ? 's' : ''} sur
            l'ensemble
            {data.total > 0 && (
              <span className="text-gray-600">
                {' '}
                — affichés {data.offset + 1} à {data.offset + shown}
              </span>
            )}
          </p>

          {shown === 0 ? (
            <p className="rounded-lg border border-gray-800 p-4 text-sm text-gray-500">
              Aucun vote sur l'ensemble ne correspond à ce filtre.
            </p>
          ) : (
            <div className="space-y-3">
              {data.items.map((vote) => (
                <VoteRow key={vote.scrutin_uid} vote={vote} selected={selected} />
              ))}
            </div>
          )}

          <div className="mt-6 flex items-center justify-between">
            <button
              onClick={() =>
                update({ offset: String(Math.max(0, offset - PAGE_SIZE)) })
              }
              disabled={offset === 0}
              className="rounded bg-gray-800 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-40"
            >
              ← Précédents
            </button>
            <button
              onClick={() => update({ offset: String(offset + PAGE_SIZE) })}
              disabled={offset + shown >= data.total}
              className="rounded bg-gray-800 px-3 py-1.5 text-sm text-gray-300 hover:bg-gray-700 disabled:cursor-not-allowed disabled:opacity-40"
            >
              Suivants →
            </button>
          </div>
        </>
      )}
    </>
  )
}
