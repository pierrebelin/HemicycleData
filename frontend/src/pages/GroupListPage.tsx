import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import { formatDate } from '../types/scrutins'
import { formatCount, type GroupListResponse, type GroupSummaryDto } from '../types/groups'

/** Pastille d'identité du groupe. La couleur est celle que l'Assemblée publie. */
export function GroupDot({ color }: { color: string | null }) {
  return (
    <span
      className="inline-block h-2.5 w-2.5 shrink-0 rounded-full border border-gray-700"
      style={{ backgroundColor: color ?? 'transparent' }}
      aria-hidden
    />
  )
}

function GroupRow({ group }: { group: GroupSummaryDto }) {
  return (
    <li className="rounded border border-gray-800 bg-gray-900/40 p-4">
      <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
        <GroupDot color={group.color} />
        <Link
          to={`/groupes/${group.uid}`}
          className="font-medium text-gray-100 hover:underline"
        >
          {group.abbrev}
        </Link>
        {group.dissolved && (
          <span className="rounded border border-gray-700 px-1.5 py-0.5 text-[10px] uppercase tracking-wide text-gray-400">
            dissous
          </span>
        )}
        {group.former_abbrevs.length > 0 && (
          <span className="text-xs text-gray-500">
            anciennement {group.former_abbrevs.join(', ')}
          </span>
        )}
      </div>

      <p className="mt-1 text-sm text-gray-400">{group.label}</p>

      <p className="mt-2 text-xs tabular-nums text-gray-500">
        {formatCount(group.member_count)} député
        {group.member_count > 1 ? 's' : ''}
        <span className="text-gray-600">
          {' '}
          au {formatDate(group.reference_date)}
        </span>
        {group.created_on && (
          <>
            {' · '}constitué le {formatDate(group.created_on)}
          </>
        )}
        {group.dissolved_on && <> · dissous le {formatDate(group.dissolved_on)}</>}
      </p>
    </li>
  )
}

/**
 * Liste des groupes parlementaires de la législature.
 *
 * Aucun taux ici, volontairement. Treize taux d'abstention alignés en colonnes
 * forment un tableau qui se lit comme un classement — ce que README.md §6
 * interdit — alors que les mêmes chiffres sur la fiche d'un seul groupe restent
 * une description. Les taux sont donc sur la fiche, une page par groupe.
 */
export default function GroupListPage() {
  const { data, isLoading, isError, error } = useQuery<GroupListResponse>({
    queryKey: ['groupes'],
    queryFn: () =>
      fetch('/api/groupes').then(async (res) => {
        if (!res.ok) throw new Error((await res.text()) || `HTTP ${res.status}`)
        return res.json()
      }),
  })

  if (isLoading) {
    return <p className="animate-pulse text-gray-400">Chargement des groupes…</p>
  }

  if (isError) {
    return (
      <div className="rounded-lg border border-red-800 bg-red-900/20 p-4">
        <p className="text-red-400">
          Erreur : {error instanceof Error ? error.message : 'inconnue'}
        </p>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold">Groupes parlementaires</h2>
        <p className="mt-1 text-sm text-gray-400">
          Les {data!.total} groupes de la {data!.groups[0]?.legislature}
          <sup>e</sup> législature. Ouvrez un groupe pour son effectif, ses
          dates et sa participation aux scrutins.
        </p>
      </div>

      <p className="rounded border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-400">
        {data!.party_note}
      </p>

      <ul className="space-y-3">
        {data!.groups.map((group) => (
          <GroupRow key={group.uid} group={group} />
        ))}
      </ul>

      <p className="rounded border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-500">
        {data!.hand_vote_note}{' '}
        <Link to="/scrutins" className="underline hover:text-gray-300">
          Voir tous les scrutins
        </Link>
      </p>
    </div>
  )
}
