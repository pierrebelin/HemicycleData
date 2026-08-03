import { Link } from 'react-router'
import type { AssignedFamilyDto } from '../types/themes'

/**
 * Chaque rattachement affiché porte son origine (RM-09) : le visiteur voit
 * si un humain a arbitré ou si la proposition automatique tient encore.
 */
export function FamilyBadges({
  families,
  linkToFamily = true,
}: {
  families: AssignedFamilyDto[]
  linkToFamily?: boolean
}) {
  if (families.length === 0) return null

  return (
    <div className="flex flex-wrap gap-1.5">
      {families.map((family) => {
        const arbitrated = family.origin === 'human_arbitration'
        const content = (
          <>
            {family.label}
            <span
              className={`ml-1.5 text-[10px] uppercase tracking-wide ${
                arbitrated ? 'text-emerald-400' : 'text-amber-400'
              }`}
              title={family.origin_note}
            >
              {arbitrated ? 'arbitré' : 'auto'}
            </span>
          </>
        )
        const className =
          'inline-flex items-center rounded border border-gray-700 bg-gray-900 px-2 py-0.5 text-xs text-gray-200'
        return linkToFamily ? (
          <Link
            key={family.code}
            to={`/themes/${family.code}`}
            className={`${className} hover:border-gray-500`}
          >
            {content}
          </Link>
        ) : (
          <span key={family.code} className={className}>
            {content}
          </span>
        )
      })}
    </div>
  )
}

/** Lacune affichée, jamais silencieuse (RM-01). */
export function UnassignedNote({ outcome }: { outcome: string | null }) {
  const reason =
    outcome === 'no_family'
      ? "Aucune famille retenue : le modèle a répondu sans rattacher ce texte."
      : outcome === 'failed'
        ? "Aucune famille : le modèle n'a pas répondu. Une nouvelle passe reprendra ce texte."
        : "Aucune famille : ce texte n'a pas encore été soumis."
  return <p className="text-xs text-gray-500">{reason}</p>
}

export function MethodNote({ note }: { note: string }) {
  return (
    <p className="rounded border border-gray-800 bg-gray-900/50 px-3 py-2 text-xs text-gray-400">
      {note}{' '}
      <Link to="/themes/methode" className="underline hover:text-gray-200">
        Méthode complète
      </Link>
    </p>
  )
}
