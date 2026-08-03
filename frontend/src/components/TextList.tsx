import { Link } from 'react-router'
import type { TextSummaryDto } from '../types/themes'
import { FamilyBadges, UnassignedNote } from './ThemeBadges'

function period(text: TextSummaryDto) {
  if (!text.first_vote || !text.last_vote) return null
  const from = new Date(text.first_vote).toLocaleDateString('fr-FR')
  const to = new Date(text.last_vote).toLocaleDateString('fr-FR')
  return from === to ? from : `${from} → ${to}`
}

export default function TextList({
  texts,
  showFamilies = true,
}: {
  texts: TextSummaryDto[]
  showFamilies?: boolean
}) {
  if (texts.length === 0) {
    return <p className="text-sm text-gray-500">Aucun texte dans cette liste.</p>
  }

  return (
    <ul className="space-y-3">
      {texts.map((text) => (
        <li
          key={text.key}
          className="rounded border border-gray-800 bg-gray-900/40 p-4"
        >
          <Link
            to={`/textes/${encodeURIComponent(text.key)}`}
            className="font-medium hover:underline"
          >
            {text.label}
          </Link>
          <p className="mt-1 text-xs text-gray-500">
            {text.scrutin_count} scrutin{text.scrutin_count > 1 ? 's' : ''}
            {period(text) ? ` · ${period(text)}` : ''}
            {text.dossier_uid ? ' · dossier rattaché' : ''}
          </p>
          <div className="mt-2">
            {showFamilies && text.families.length > 0 ? (
              <FamilyBadges families={text.families} />
            ) : (
              <UnassignedNote outcome={text.last_attempt_outcome} />
            )}
          </div>
        </li>
      ))}
    </ul>
  )
}
