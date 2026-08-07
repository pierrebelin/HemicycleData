import { Link } from 'react-router'
import type { ScrutinSummaryDto } from '../types/scrutins'
import { formatDateShort } from '../types/scrutins'
import { ListCard, Meta, Note, Pill, TallyLine, VoteBar } from './ui'

export { TallyLine } from './ui'

/**
 * Lacune de couverture affichée telle quelle : les votes à main levée sont
 * absents de la source, le site ne peut rien en dire.
 */
export function CoverageNote({ note }: { note: string }) {
  return <Note>{note}</Note>
}

/**
 * Ce que le scrutin porte, et sous quelle forme il a été mis aux voix.
 *
 * L'intitulé publié par l'Assemblée est procédural avant d'être informatif :
 * « l'ensemble de la proposition de loi visant à moderniser la gestion du
 * patrimoine immobilier de l'État (texte de la commission mixte paritaire) ».
 * Le titre du dossier dit la même chose en six mots. On lit donc le dossier en
 * premier et l'intitulé juste dessous — jamais à sa place : c'est l'intitulé
 * qui fait foi, et lui seul distingue un vote sur l'ensemble d'un vote sur un
 * amendement du même texte.
 */
function titles(scrutin: ScrutinSummaryDto) {
  if (scrutin.dossier_label) {
    return { lead: scrutin.dossier_label, official: scrutin.subject }
  }
  return { lead: scrutin.subject, official: null }
}

export function ScrutinRow({ scrutin }: { scrutin: ScrutinSummaryDto }) {
  const { lead, official } = titles(scrutin)

  return (
    <Link
      to={`/scrutins/${scrutin.uid}`}
      className="group flex flex-col gap-3 px-4 py-3 transition-colors hover:bg-surface-soft sm:flex-row sm:items-center sm:gap-6"
    >
      <div className="min-w-0 flex-1">
        <div className="mb-1 flex flex-wrap items-center gap-x-2 gap-y-1">
          {/* Le code — « adopté », « rejeté » — plutôt que le libellé, qui est
              une phrase entière (« L'Assemblée nationale a adopté… ») et ne
              tient pas dans une pastille. La phrase reste en infobulle. */}
          <Pill
            tone={scrutin.outcome_code === 'adopté' ? 'yes' : 'no'}
            title={scrutin.outcome_label}
          >
            {scrutin.outcome_code}
          </Pill>
          <Meta>
            n° {scrutin.number} · {formatDateShort(scrutin.date)}
          </Meta>
          {scrutin.has_reconstructed_tallies && (
            <Pill
              tone="info"
              title="La source ne publie pas les groupes sur ce scrutin : la répartition est reconstituée."
            >
              répartition reconstituée
            </Pill>
          )}
        </div>

        <p className="text-[15px] font-semibold leading-snug text-ink transition-colors group-hover:text-accent">
          {lead}
        </p>
        {official && (
          <p className="mt-0.5 line-clamp-1 text-xs leading-relaxed text-ink-faint">
            {official}
          </p>
        )}
      </div>

      {/* Colonne de largeur fixe, pour que les barres s'alignent d'une ligne à
          l'autre. Chaque barre se normalise sur son propre scrutin : elle donne
          la répartition, pas le nombre de votants — deux scrutins n'ont pas le
          même nombre de présents et la longueur ne le dirait pas. */}
      <div className="shrink-0 space-y-1.5 sm:w-52">
        <VoteBar tally={scrutin.tally} />
        <TallyLine tally={scrutin.tally} dense />
      </div>
    </Link>
  )
}

export default function ScrutinList({
  scrutins,
}: {
  scrutins: ScrutinSummaryDto[]
}) {
  return (
    <ListCard>
      {scrutins.map((scrutin) => (
        <ScrutinRow key={scrutin.uid} scrutin={scrutin} />
      ))}
    </ListCard>
  )
}
