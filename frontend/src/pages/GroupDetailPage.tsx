import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from 'react-router'
import { formatDate } from '../types/scrutins'
import {
  formatCount,
  formatPerMille,
  type GroupDetailResponse,
  type ParticipationRatesDto,
} from '../types/groups'
import { GroupDot } from './GroupListPage'

/**
 * Couleurs des trois parts.
 *
 * Ni vert ni rouge, contrairement aux pages de vote : là-bas ils disent « pour »
 * et « contre », ici ils diraient « bien » et « mal ». Une participation élevée
 * n'est pas un mérite et une abstention n'est pas une faute (PROJECT.md §6).
 */
const RATE_BARS = {
  expressed: 'bg-sky-500',
  abstention: 'bg-amber-500',
  absence: 'bg-slate-500',
} as const

function Section({
  title,
  children,
}: {
  title: string
  children: React.ReactNode
}) {
  return (
    <section className="rounded border border-gray-800 bg-gray-900/40 p-4">
      <h3 className="text-sm font-medium text-gray-300">{title}</h3>
      <div className="mt-3">{children}</div>
    </section>
  )
}

function Fact({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-0.5 border-b border-gray-800/70 py-1.5 last:border-0">
      <span className="text-xs text-gray-500">{label}</span>
      <span className="text-sm tabular-nums text-gray-200">{children}</span>
    </div>
  )
}

function Note({ children }: { children: React.ReactNode }) {
  return (
    <p className="rounded border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-500">
      {children}
    </p>
  )
}

/** Barre des trois parts. Elle rend visible ce que le pourcentage résume. */
function RateBar({ rates }: { rates: ParticipationRatesDto }) {
  return (
    <div className="flex h-2 w-full overflow-hidden rounded bg-gray-800">
      <div
        className={RATE_BARS.expressed}
        style={{ width: `${rates.expressed_per_mille / 10}%` }}
      />
      <div
        className={RATE_BARS.abstention}
        style={{ width: `${rates.abstention_per_mille / 10}%` }}
      />
      <div
        className={RATE_BARS.absence}
        style={{ width: `${rates.absence_per_mille / 10}%` }}
      />
    </div>
  )
}

/**
 * Un taux et le compte brut qui l'a produit. Le pourcentage ne remplace jamais
 * le chiffre : les deux sont affichés côte à côte (PROJECT.md §6).
 */
function Rate({
  label,
  swatch,
  perMille,
  count,
  detail,
}: {
  label: string
  swatch: string
  perMille: number
  count: number
  detail?: string
}) {
  return (
    <div className="flex flex-wrap items-baseline gap-x-2 gap-y-0.5 border-b border-gray-800/70 py-1.5 last:border-0">
      <span className={`inline-block h-2 w-2 shrink-0 rounded-sm ${swatch}`} aria-hidden />
      <span className="text-xs text-gray-400">{label}</span>
      <span className="ml-auto text-sm font-medium tabular-nums text-gray-100">
        {formatPerMille(perMille)}
      </span>
      <span className="w-full pl-4 text-xs tabular-nums text-gray-600">
        {formatCount(count)} position{count > 1 ? 's' : ''}
        {detail && <> — {detail}</>}
      </span>
    </div>
  )
}

/**
 * Fiche d'un groupe parlementaire.
 *
 * Les taux décomposent la participation — voix exprimées, abstention,
 * non-participation — et jamais le sens du vote. « Pour » et « contre » sont
 * additionnés ensemble : cumulés sur toute la législature, dont 86 % de
 * scrutins d'amendement, ils ne décriraient aucune position et se liraient
 * pourtant comme une (PROJECT.md §6).
 */
export default function GroupDetailPage() {
  const { uid } = useParams<{ uid: string }>()

  const { data, isLoading, isError, error } = useQuery<GroupDetailResponse>({
    queryKey: ['groupes', uid],
    queryFn: () =>
      fetch(`/api/groupes/${encodeURIComponent(uid!)}`).then(async (res) => {
        if (!res.ok) throw new Error((await res.text()) || `HTTP ${res.status}`)
        return res.json()
      }),
  })

  if (isLoading) {
    return <p className="animate-pulse text-gray-400">Chargement du groupe…</p>
  }

  if (isError) {
    return (
      <div className="space-y-3">
        <div className="rounded-lg border border-red-800 bg-red-900/20 p-4">
          <p className="text-red-400">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
        <Link to="/groupes" className="text-sm text-gray-400 underline hover:text-gray-200">
          ← Tous les groupes
        </Link>
      </div>
    )
  }

  const group = data!
  const { counts, rates } = group

  return (
    <div className="space-y-6">
      <div>
        <Link to="/groupes" className="text-xs text-gray-500 underline hover:text-gray-300">
          ← Tous les groupes
        </Link>
        <div className="mt-2 flex flex-wrap items-baseline gap-x-2 gap-y-1">
          <GroupDot color={group.color} />
          <h2 className="text-xl font-bold">{group.abbrev}</h2>
          {group.dissolved && (
            <span className="rounded border border-gray-700 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-gray-400">
              dissous
            </span>
          )}
        </div>
        <p className="mt-1 text-sm text-gray-400">{group.label}</p>
      </div>

      <Note>{group.party_note}</Note>

      <Section title="Identité">
        <Fact label="Législature">{group.legislature}<sup>e</sup></Fact>
        <Fact label="Constitué le">
          {group.created_on ? formatDate(group.created_on) : 'non publié'}
        </Fact>
        {group.dissolved_on && (
          <Fact label="Dissous le">{formatDate(group.dissolved_on)}</Fact>
        )}
        {group.former_abbrevs.length > 0 && (
          <Fact label="Sigles antérieurs">{group.former_abbrevs.join(', ')}</Fact>
        )}
      </Section>

      <Section title="Effectif">
        <Fact label={`Députés au ${formatDate(group.reference_date)}`}>
          {formatCount(group.member_count)}
        </Fact>
        <Fact label="Députés y ayant siégé depuis sa constitution">
          {formatCount(group.total_member_count)}
        </Fact>
        {group.published_member_range && (
          <Fact label="Effectif publié sur les scrutins">
            {group.published_member_range.stable
              ? formatCount(group.published_member_range.min)
              : `de ${formatCount(group.published_member_range.min)} à ${formatCount(
                  group.published_member_range.max,
                )}`}
          </Fact>
        )}
        {group.qualities.map((quality) => (
          <Fact key={quality.quality} label={quality.quality}>
            {formatCount(quality.members)}
          </Fact>
        ))}
      </Section>

      <Section title="Présence dans les scrutins">
        <Fact label="Scrutins portant une ligne pour ce groupe">
          {formatCount(group.line_count)}
        </Fact>
        {group.first_scrutin_date && group.last_scrutin_date && (
          <Fact label="Du premier au dernier scrutin">
            {formatDate(group.first_scrutin_date)} → {formatDate(group.last_scrutin_date)}
          </Fact>
        )}
        {group.reconstructed_count > 0 && (
          <Fact label="Répartitions reconstituées">
            {formatCount(group.reconstructed_count)}
          </Fact>
        )}
        {group.silent_line_count > 0 && (
          <Fact label="Scrutins sans aucun membre du groupe">
            {formatCount(group.silent_line_count)}
          </Fact>
        )}
      </Section>

      <Section title="Participation">
        {rates === null ? (
          <p className="text-sm text-gray-500">
            L'Assemblée ne publie aucune position pour ce groupe : il n'y a
            aucun taux à calculer.
          </p>
        ) : (
          <>
            <RateBar rates={rates} />
            <div className="mt-3">
              <Rate
                label="Voix exprimées"
                swatch={RATE_BARS.expressed}
                perMille={rates.expressed_per_mille}
                count={counts.expressed}
                detail="pour et contre confondus"
              />
              <Rate
                label="Abstention"
                swatch={RATE_BARS.abstention}
                perMille={rates.abstention_per_mille}
                count={counts.abstentions}
              />
              <Rate
                label="Non-participation"
                swatch={RATE_BARS.absence}
                perMille={rates.absence_per_mille}
                count={counts.not_voting + counts.voluntary_not_voting}
                detail={`dont ${formatCount(
                  counts.voluntary_not_voting,
                )} non-votant${counts.voluntary_not_voting > 1 ? 's' : ''} volontaire${
                  counts.voluntary_not_voting > 1 ? 's' : ''
                }`}
              />
            </div>
            <p className="mt-3 text-xs tabular-nums text-gray-500">
              Rapportés à {formatCount(counts.published_positions)} positions
              publiées, sur {formatCount(group.line_count)} scrutin
              {group.line_count > 1 ? 's' : ''}.
            </p>
          </>
        )}
      </Section>

      <div className="space-y-1.5">
        <Note>{group.rate_note}</Note>
        <Note>{group.comparison_note}</Note>
        <Note>{group.hand_vote_note}</Note>
        {group.reconstructed_count > 0 && <Note>{group.reconstructed_note}</Note>}
        {group.silent_line_count > 0 && <Note>{group.silent_line_note}</Note>}
      </div>

      <p className="text-sm">
        <Link
          to={`/votes-par-groupe?groupes=${encodeURIComponent(group.abbrev)}`}
          className="text-gray-400 underline hover:text-gray-200"
        >
          Voir les votes de {group.abbrev}, scrutin par scrutin →
        </Link>
      </p>
    </div>
  )
}
