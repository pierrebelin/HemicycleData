import { useMemo, useState } from 'react'
import type { TallyDto } from '../types/scrutins'
import { positionLabels } from '../types/scrutins'
import { PLAN_SIEGES, PLAN_VUE, RAYON_SIEGE } from './planHemicycle'
import {
  positions,
  REMPLISSAGE_PAR_POSITION,
  TEXTE_PAR_POSITION,
  type SiegeVote,
} from './positions'

/** Au-delà, le pointeur n'est sur personne : nommer le siège le plus proche
 *  ferait dire à l'infobulle ce que le lecteur ne vise pas. */
const PORTEE_SURVOL = 2.4

type Place = { numero: number; x: number; y: number; vote: SiegeVote | null }

/**
 * Infobulle d'un siège. Le groupe est nommé tel que l'Assemblée le publie,
 * sigle et libellé — jamais un parti (PROJECT.md §3.1).
 */
function Infobulle({ place }: { place: Place }) {
  // Un siège du haut de l'arc n'a pas la place d'accueillir l'infobulle
  // au-dessus de lui : elle sortirait du cadre.
  const dessus = place.y > PLAN_VUE.hauteur * 0.3
  const gauche = Math.min(Math.max((place.x / PLAN_VUE.largeur) * 100, 15), 85)
  const vote = place.vote

  return (
    <div
      className={`pointer-events-none absolute z-10 w-max max-w-[15rem] -translate-x-1/2 rounded-lg border border-line bg-raised px-3 py-2 shadow-lg ${
        dessus ? '-translate-y-[calc(100%+0.75rem)]' : 'translate-y-3'
      }`}
      style={{
        left: `${gauche}%`,
        top: `${(place.y / PLAN_VUE.hauteur) * 100}%`,
      }}
    >
      {vote ? (
        <>
          <p className="text-sm font-medium leading-tight text-ink">
            {vote.full_name ?? vote.actor_uid}
          </p>
          {vote.groupAbbrev && (
            <p className="mt-0.5 text-xs leading-tight text-ink-3">
              {vote.groupAbbrev}
              {vote.groupLabel && (
                <span className="text-ink-4"> — {vote.groupLabel}</span>
              )}
            </p>
          )}
          <p className="mt-1.5 text-xs">
            <span className={TEXTE_PAR_POSITION[vote.position]}>
              {positionLabels[vote.position]}
            </span>
            {vote.by_delegation && (
              <span className="text-ink-4"> · par délégation</span>
            )}
            <span className="tabular-nums text-ink-5">
              {' '}
              · siège {place.numero}
            </span>
          </p>
        </>
      ) : (
        <>
          <p className="text-sm font-medium leading-tight text-ink tabular-nums">
            Siège {place.numero}
          </p>
          {/* La source ne publie pas qui occupe un siège resté muet : on ne
              peut donc pas nommer le député, seulement constater l'absence. */}
          <p className="mt-0.5 text-xs leading-tight text-ink-3">
            Aucune position enregistrée sur ce scrutin
          </p>
        </>
      )}
    </div>
  )
}

/**
 * Les 582 sièges de l'hémicycle, chacun coloré par la position du député qui
 * l'occupe. Un siège dont aucune position n'a été enregistrée reste dessiné,
 * en gris : le taire ferait passer une absence pour un hémicycle plus petit.
 *
 * Le placement vient du plan publié par l'Assemblée (voir `planHemicycle.ts`),
 * pas d'une disposition reconstituée : la source publie le numéro de siège avec
 * chaque position de vote, et le plan donne l'emplacement de ce numéro. Colorer
 * par position sans tenir compte du siège produirait un dessin lisible mais
 * faux — des blocs nets là où la réalité montre les fractures à l'intérieur des
 * groupes.
 *
 * Le survol nomme le député. La cible est le siège le plus proche du pointeur,
 * pas le cercle exactement sous lui : à ce diamètre, exiger le contact rendrait
 * la moitié des sièges impossibles à atteindre. La liste nominale complète
 * reste sur la page du scrutin — le survol l'illustre, il ne la remplace pas.
 */
export default function Hemicycle({
  votes,
  labelledBy,
}: {
  votes: SiegeVote[]
  labelledBy?: string
}) {
  const [survol, setSurvol] = useState<number | null>(null)

  const places = useMemo<Place[]>(() => {
    const parSiege = new Map<number, SiegeVote>()
    votes.forEach((vote) => {
      if (vote.seat !== null) parSiege.set(vote.seat, vote)
    })
    return Object.entries(PLAN_SIEGES).map(([numero, [x, y]]) => ({
      numero: Number(numero),
      x,
      y,
      vote: parSiege.get(Number(numero)) ?? null,
    }))
  }, [votes])

  /** Positions que le plan ne sait pas placer : dites, jamais tues (§2). */
  const horsPlan = useMemo(
    () => votes.filter((v) => v.seat === null || !(v.seat in PLAN_SIEGES)).length,
    [votes],
  )

  const parRemplissage = useMemo(() => {
    const table = new Map<string, Place[]>()
    places.forEach((place) => {
      const classe = place.vote
        ? REMPLISSAGE_PAR_POSITION[place.vote.position]
        : 'fill-seat-empty'
      const lot = table.get(classe) ?? []
      lot.push(place)
      table.set(classe, lot)
    })
    return table
  }, [places])

  function viser(event: React.PointerEvent<SVGSVGElement>) {
    const cadre = event.currentTarget.getBoundingClientRect()
    const x = ((event.clientX - cadre.left) / cadre.width) * PLAN_VUE.largeur
    const y = ((event.clientY - cadre.top) / cadre.height) * PLAN_VUE.hauteur

    let plusProche = -1
    let ecart = Infinity
    places.forEach((place, index) => {
      const d = (place.x - x) ** 2 + (place.y - y) ** 2
      if (d < ecart) {
        ecart = d
        plusProche = index
      }
    })

    const portee = RAYON_SIEGE * PORTEE_SURVOL
    setSurvol(ecart <= portee ** 2 ? plusProche : null)
  }

  const vise = survol !== null ? places[survol] : null

  return (
    <div className="relative">
      <svg
        viewBox={`0 0 ${PLAN_VUE.largeur} ${PLAN_VUE.hauteur}`}
        className="block w-full"
        role="img"
        aria-labelledby={labelledBy}
        onPointerMove={viser}
        onPointerLeave={() => setSurvol(null)}
      >
        {[...parRemplissage].map(([classe, lot]) => (
          <g key={classe} className={classe}>
            {lot.map((place) => (
              <circle
                key={place.numero}
                cx={place.x}
                cy={place.y}
                r={RAYON_SIEGE}
              />
            ))}
          </g>
        ))}

        {vise && (
          <circle
            cx={vise.x}
            cy={vise.y}
            r={RAYON_SIEGE + 1.4}
            className="fill-none stroke-ink"
            strokeWidth={0.8}
          />
        )}
      </svg>

      {vise && <Infobulle place={vise} />}

      {horsPlan > 0 && (
        <p className="mt-1 text-xs text-ink-4">
          {horsPlan} position{horsPlan > 1 ? 's' : ''} sans siège localisable sur
          le plan {horsPlan > 1 ? 'ne sont pas représentées' : "n'est pas représentée"}.
        </p>
      )}
    </div>
  )
}

/** Barre de répartition, pour les cartes où l'arc serait trop encombrant. */
export function BarreRepartition({ tally }: { tally: TallyDto }) {
  const blocs = positions(tally)
  const total = blocs.reduce((somme, bloc) => somme + bloc.total, 0)
  if (total === 0) return null

  return (
    <div className="flex h-1.5 w-full overflow-hidden rounded-full bg-sunken">
      {blocs
        .filter((bloc) => bloc.total > 0)
        .map((bloc) => (
          <div
            key={bloc.cle}
            className={bloc.fond}
            style={{ width: `${(bloc.total / total) * 100}%` }}
            title={`${bloc.libelle} : ${bloc.total}`}
          />
        ))}
    </div>
  )
}
