import { keepPreviousData, useQuery } from '@tanstack/react-query'
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

/** Plafond supposé tant que la réponse n'a pas donné le sien. */
const MAX_GROUPS_BEFORE_LOAD = 4

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
        // Deux colonnes au plus par ligne : à quatre groupes de front, les
        // cartes seraient trop étroites pour porter les chiffres bruts, et il
        // ne resterait que les pourcentages (PROJECT.md §6).
        <div
          className={`mt-3 grid gap-2 ${selected.length > 1 ? 'sm:grid-cols-2' : ''}`}
        >
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

/**
 * Sélecteur des groupes comparés. Une pastille par groupe du référentiel :
 * l'offre reste entière et visible, et la limite est annoncée avant d'être
 * atteinte plutôt que signalée par une erreur du serveur.
 */
function GroupPicker({
  groups,
  selected,
  max,
  onToggle,
}: {
  groups: GroupDto[]
  selected: string[]
  max: number
  onToggle: (abbrev: string) => void
}) {
  const full = selected.length >= max

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-baseline gap-2">
        <span className="text-xs text-gray-500">Groupes comparés</span>
        <span className="text-xs tabular-nums text-gray-600">
          {selected.length} sur {max} au maximum
        </span>
        {selected.length === 0 && (
          <span className="text-xs text-gray-600">
            — choisissez les groupes à comparer
          </span>
        )}
      </div>
      <div className="flex flex-wrap gap-1.5">
        {groups.map((group) => {
          const isSelected = selected.includes(group.abbrev)
          return (
            <button
              key={group.uid}
              type="button"
              onClick={() => onToggle(group.abbrev)}
              aria-pressed={isSelected}
              disabled={full && !isSelected}
              title={group.label}
              className={`flex items-center gap-1.5 rounded border px-2 py-1 text-sm transition-colors ${
                isSelected
                  ? 'border-gray-600 bg-gray-800 text-gray-100'
                  : 'border-gray-800 bg-gray-900 text-gray-400 hover:border-gray-700 hover:text-gray-200 disabled:cursor-not-allowed disabled:opacity-40 disabled:hover:border-gray-800 disabled:hover:text-gray-400'
              }`}
            >
              <span
                className="inline-block h-2.5 w-2.5 shrink-0 rounded-full border border-gray-700"
                style={{ backgroundColor: group.color ?? 'transparent' }}
                aria-hidden
              />
              {group.abbrev}
            </button>
          )
        })}
      </div>
    </div>
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

  // Aucune sélection par défaut : présélectionner des groupes reviendrait à
  // désigner la comparaison qui mérite d'être faite, et à orienter la lecture
  // avant qu'elle commence (PROJECT.md §6).
  const groupsParam = params.get('groupes') ?? ''
  const theme = params.get('theme') ?? ''
  const offset = Math.max(0, Number(params.get('offset') ?? 0) || 0)

  const requested = groupsParam
    .split(',')
    .map((token) => token.trim())
    .filter(Boolean)

  // Sans groupe demandé, la page n'affiche aucun vote : la requête ne sert
  // alors qu'à ramener le référentiel des groupes du sélecteur, et une page
  // entière de votes serait chargée pour rien.
  const query = new URLSearchParams({
    groupes: groupsParam,
    limit: String(requested.length > 0 ? PAGE_SIZE : 1),
    offset: String(offset),
  })
  if (theme) query.set('theme', theme)

  const { data, isLoading, isError, error, isPlaceholderData } =
    useQuery<FinalVoteListResponse>({
      queryKey: ['votes-finaux', query.toString()],
      queryFn: () =>
        fetch(`/api/votes-finaux?${query}`).then(async (res) => {
          if (!res.ok) throw new Error((await res.text()) || `HTTP ${res.status}`)
          return res.json()
        }),
      // Changer de groupes ne doit pas vider le haut de la page : le référentiel
      // du sélecteur voyage dans la même réponse, et le laisser disparaître
      // ferait clignoter les pastilles au moment même où on clique dessus. La
      // réponse précédente tient le décor pendant que les votes se rechargent.
      placeholderData: keepPreviousData,
    })

  const families = useQuery<FamiliesResponse>({
    queryKey: ['themes'],
    queryFn: () =>
      fetch('/api/themes').then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

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

  const groups = data?.groups ?? []
  const selected = data?.selected ?? []
  const shown = data?.items.length ?? 0
  const maxGroups = data?.max_compared_groups ?? MAX_GROUPS_BEFORE_LOAD

  // Le sigle affiché fait foi : un groupe renommé est proposé sous son sigle
  // courant, et c'est celui-là qui part dans l'adresse.
  // Pendant un rechargement, la réponse en main porte encore l'ancienne
  // sélection : c'est l'adresse qui fait foi, sinon la pastille qu'on vient de
  // cliquer resterait éteinte jusqu'à l'arrivée des votes.
  const selectedAbbrevs =
    data === undefined || isPlaceholderData
      ? requested
      : selected.map((group) => group.abbrev)
  const hasSelection = selectedAbbrevs.length > 0
  // Les votes affichés ne correspondent plus à la sélection : la zone se
  // recharge, le reste de la page ne bouge pas.
  const reloading = hasSelection && (isLoading || isPlaceholderData) && !isError

  function toggleGroup(abbrev: string) {
    const next = selectedAbbrevs.includes(abbrev)
      ? selectedAbbrevs.filter((token) => token !== abbrev)
      : [...selectedAbbrevs, abbrev].slice(0, maxGroups)
    // Retirer le dernier groupe ramène à l'état de départ : l'adresse perd le
    // paramètre, et la page repose la question au lieu de retomber sur une
    // sélection qu'elle aurait choisie seule.
    update({ groupes: next.join(',') })
  }

  return (
    <>
      <div className="mb-6">
        <h2 className="mb-1 text-xl font-semibold">Votes par groupe</h2>
      </div>

      <div className="mb-6 space-y-3">
        <GroupPicker
          groups={groups}
          selected={selectedAbbrevs}
          max={maxGroups}
          onToggle={toggleGroup}
        />

        <div className="flex flex-wrap items-center gap-4">
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

        {data && hasSelection && (
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
            {(isPlaceholderData ? [] : selected)
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

      {!hasSelection && (
        <div className="rounded-lg border border-gray-800 bg-gray-900/50 p-4">
          <p className="text-sm text-gray-300">
            Aucun groupe n'est sélectionné au départ.
          </p>
          <p className="mt-2 text-sm text-gray-500">
            Choisir pour vous les groupes à comparer reviendrait à désigner la
            comparaison qui mérite d'être faite. Sélectionnez ci-dessus les
            groupes que vous voulez lire côte à côte — {maxGroups} au maximum —
            pour afficher les votes sur l'ensemble.
          </p>
        </div>
      )}

      {reloading && (
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

      {data && hasSelection && !reloading && (
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
