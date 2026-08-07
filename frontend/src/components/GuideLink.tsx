import { Link } from 'react-router'

/**
 * Renvoi vers le guide de lecture. La prose vit dans `/comprendre` et nulle
 * part ailleurs : une explication recopiée sur une page de vote finirait par
 * contredire la page qui fait référence.
 *
 * Le niveau de lecture n'est pas passé dans l'URL — il est mémorisé par la
 * page « Comprendre » elle-même, pour que ce lien reste partageable tel quel.
 */
export default function GuideLink({
  ancre,
  children,
}: {
  ancre: string
  children: React.ReactNode
}) {
  return (
    <Link
      to={`/comprendre#${ancre}`}
      className="text-xs text-ink-4 underline decoration-line-strong underline-offset-2 hover:text-ink-2 hover:decoration-ink-4"
    >
      {children}
    </Link>
  )
}
