import type { OutcomeDto, OutcomeKind } from '../types/dossiers'

const styles: Record<OutcomeKind, string> = {
  promulgated: 'bg-emerald-900/30 border-emerald-800/50 text-emerald-300',
  rejected: 'bg-rose-900/30 border-rose-800/50 text-rose-300',
  withdrawn: 'bg-gray-800 border-gray-700 text-gray-400',
  merged_into: 'bg-gray-800 border-gray-700 text-gray-400',
  no_recorded_conclusion: 'bg-gray-800 border-gray-700 text-gray-400',
}

function formatDate(iso: string) {
  return new Date(iso + 'T00:00:00').toLocaleDateString('fr-FR', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  })
}

/**
 * Pastille de sort du dossier.
 *
 * Rien n'est affiché quand la source ne conclut rien : sur les neuf dixièmes
 * des dossiers, une pastille « en cours » serait une affirmation que personne
 * n'a écrite. Le dernier acte, déjà présent sur la carte, dit ce qui est su.
 */
export default function OutcomeBadge({ outcome }: { outcome: OutcomeDto }) {
  if (outcome.kind === 'no_recorded_conclusion') return null

  return (
    <span
      className={`text-xs px-1.5 py-0.5 rounded border ${styles[outcome.kind]}`}
    >
      {outcome.label}
      {outcome.date && ` — ${formatDate(outcome.date)}`}
    </span>
  )
}

/**
 * Bloc détaillé, pour la page d'un dossier. Contrairement à la pastille, il
 * dit explicitement quand la source ne conclut rien plutôt que de se taire.
 */
export function OutcomePanel({
  outcome,
  lastActivityLabel,
  lastActivityDate,
}: {
  outcome: OutcomeDto
  lastActivityLabel: string
  lastActivityDate: string
}) {
  if (outcome.kind === 'no_recorded_conclusion') {
    return (
      <div className="bg-gray-900 border border-gray-800 rounded-lg p-4">
        <p className="text-sm text-gray-300">Sans conclusion enregistrée</p>
        <p className="text-xs text-gray-500 mt-1">
          L'Assemblée n'a publié aucun acte de clôture pour ce dossier. Dernier
          acte connu : {lastActivityLabel}, le {formatDate(lastActivityDate)}.
        </p>
      </div>
    )
  }

  return (
    <div className="bg-gray-900 border border-gray-800 rounded-lg p-4">
      <div className="flex items-center gap-2">
        <OutcomeBadge outcome={outcome} />
        {!outcome.is_final && (
          <span className="text-xs text-gray-500">
            la navette peut reprendre
          </span>
        )}
      </div>

      {outcome.law_code && (
        <p className="text-xs text-gray-400 mt-2">
          Loi n° {outcome.law_code}
          {outcome.law_jo_date &&
            `, publiée au Journal officiel le ${formatDate(outcome.law_jo_date)}`}
          {outcome.legifrance_url && (
            <>
              {' — '}
              <a
                href={outcome.legifrance_url}
                target="_blank"
                rel="noreferrer"
                className="text-blue-400 hover:underline"
              >
                texte sur Légifrance
              </a>
            </>
          )}
        </p>
      )}

      {outcome.merged_into_uid && (
        <p className="text-xs text-gray-400 mt-2">
          Suite de l'examen dans le dossier{' '}
          <a
            href={`/dossiers/${outcome.merged_into_uid}`}
            className="text-blue-400 hover:underline"
          >
            {outcome.merged_into_uid}
          </a>
        </p>
      )}
    </div>
  )
}
