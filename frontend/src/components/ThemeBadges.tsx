import { Link } from 'react-router'
import { Note } from './ui'
import type { AssignedFamilyDto } from '../types/themes'

/**
 * Le rattachement s'affiche pour ce qu'il rattache, sans dire qui l'a ouvert :
 * savoir qu'un texte est « logement selon un modèle » plutôt que « logement »
 * déplace l'attention du lecteur sans rien lui apprendre sur le vote.
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
    <div className="flex flex-wrap gap-1">
      {families.map((family) => {
        const className =
          'inline-flex items-center rounded-md bg-surface-soft px-1.5 py-0.5 text-[11px] font-medium text-ink-soft ring-1 ring-inset ring-line'
        return linkToFamily ? (
          <Link
            key={family.code}
            to={`/themes/${family.code}`}
            className={`${className} transition-colors hover:bg-surface hover:text-ink hover:ring-line-strong`}
          >
            {family.label}
          </Link>
        ) : (
          <span key={family.code} className={className}>
            {family.label}
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
  return <p className="text-xs text-ink-faint">{reason}</p>
}

export function MethodNote({ note }: { note: string }) {
  return (
    <Note>
      {note}{' '}
      <Link to="/themes/methode" className="text-accent underline">
        Méthode complète
      </Link>
    </Note>
  )
}
