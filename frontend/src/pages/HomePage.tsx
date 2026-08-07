import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import Hemicycle, { BarreRepartition } from '../components/Hemicycle'
import {
  positions,
  siegesDesGroupes,
  tallyDesVotes,
} from '../components/positions'
import type {
  ScrutinDetailDto,
  ScrutinListResponse,
  ScrutinSummaryDto,
} from '../types/scrutins'
import { formatDate } from '../types/scrutins'

const DERNIERS = 3

/**
 * Ce que le site refuse de faire, énoncé à l'accueil plutôt qu'enfoui dans une
 * page annexe (PROJECT.md §1, §6) : c'est la contrepartie qui rend la promesse
 * vérifiable.
 */
const ANTI_PROMESSES = [
  'Aucune note, aucun classement, aucun score attribué à un groupe ou à une personne.',
  "Aucune évaluation d'un texte : ni bon, ni mauvais, ni efficace.",
  "Aucune traduction d'un groupe parlementaire en parti politique.",
  'Aucun chiffre produit par un modèle de langage.',
  'Aucune prédiction électorale, aucune comparaison de programmes.',
]

const PRINCIPE = [
  {
    titre: 'Tout est ingéré',
    texte:
      "Tous les scrutins publiés par l'Assemblée entrent dans la base, sans sélection. Le tri ordonne l'affichage, il ne filtre jamais. Une lacune connue est affichée comme telle.",
  },
  {
    titre: 'Les textes sont rattachés à des thèmes',
    texte:
      "C'est le seul endroit où un jugement entre dans le produit. La méthode est publiée, chaque rattachement porte son origine, et il reste révisable.",
  },
  {
    titre: 'Les chiffres sont affichés bruts',
    texte:
      "Des nombres, pas d'adverbes. La position de chaque député est consultable au détail du scrutin, et chaque page renvoie à la source officielle.",
  },
]


function formatNombre(valeur: number) {
  return valeur.toLocaleString('fr-FR')
}


function Section({
  children,
  className = '',
}: {
  children: React.ReactNode
  className?: string
}) {
  return (
    <section className={className}>
      <div className="mx-auto w-full max-w-6xl px-6">{children}</div>
    </section>
  )
}

function TitreSection({
  surtitre,
  titre,
  chapeau,
}: {
  surtitre: string
  titre: string
  chapeau?: string
}) {
  return (
    <div className="max-w-2xl">
      <p className="text-xs font-semibold uppercase tracking-[0.14em] text-ink-4">
        {surtitre}
      </p>
      <h2 className="mt-2 text-2xl font-bold tracking-tight sm:text-3xl">
        {titre}
      </h2>
      {chapeau && (
        <p className="mt-3 text-sm leading-relaxed text-ink-3">{chapeau}</p>
      )}
    </div>
  )
}


/** Carte de scrutin propre à l'accueil : la barre remplace le détail chiffré. */
function CarteScrutin({ scrutin }: { scrutin: ScrutinSummaryDto }) {
  const adopte = scrutin.outcome_code === 'adopté'
  return (
    <Link
      to={`/scrutins/${scrutin.uid}`}
      className="group flex flex-col rounded-xl border border-line bg-raised p-5 transition-colors hover:border-line-stronger"
    >
      <div className="flex items-center justify-between gap-3">
        <span
          className={`inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-medium ${
            adopte
              ? 'border-for-line bg-for-soft text-for-ink'
              : 'border-against-line bg-against-soft text-against-ink'
          }`}
        >
          {scrutin.outcome_code}
        </span>
        <span className="text-xs text-ink-4">{formatDate(scrutin.date)}</span>
      </div>

      <p className="mt-3 line-clamp-3 flex-1 text-sm leading-snug text-ink-1 group-hover:text-ink">
        {scrutin.subject}
      </p>

      <div className="mt-4">
        <BarreRepartition tally={scrutin.tally} />
        <div className="mt-2 flex flex-wrap gap-x-3 gap-y-1 text-xs tabular-nums">
          {positions(scrutin.tally)
            .filter((bloc) => bloc.total > 0)
            .map((bloc) => (
              <span key={bloc.cle} className={bloc.texte}>
                {formatNombre(bloc.total)}{' '}
                <span className="text-ink-4">{bloc.libelleAccorde}</span>
              </span>
            ))}
        </div>
      </div>

      <p className="mt-3 text-xs text-ink-4">
        n° {scrutin.number} · {scrutin.ballot_type}
      </p>
    </Link>
  )
}

export default function HomePage() {
  const scrutins = useQuery({
    queryKey: ['scrutins', 'accueil'],
    queryFn: (): Promise<ScrutinListResponse> =>
      fetch(`/api/scrutins?limit=${DERNIERS}&offset=0`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  const dernier = scrutins.data?.scrutins[0]

  // Le résumé de liste ne porte que les totaux. Placer chaque vote sur son
  // siège demande les positions nominales, donc le détail du scrutin.
  const detail = useQuery({
    queryKey: ['scrutin', dernier?.uid],
    enabled: Boolean(dernier),
    queryFn: (): Promise<ScrutinDetailDto> =>
      fetch(`/api/scrutins/${dernier!.uid}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  const votesNominaux = detail.data && siegesDesGroupes(detail.data.groups)

  return (
    <div className="pb-20">
      {/* ---------------------------------------------------------------- Hero */}
      <Section className="border-b border-line bg-gradient-to-b from-sunken to-surface pb-16 pt-14 sm:pt-20">
        <div className="grid items-center gap-12 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
          <div>
            <p className="text-xs font-semibold uppercase tracking-[0.14em] text-ink-4">
              Assemblée nationale
              {detail.data && <> · {detail.data.legislature}ᵉ législature</>}
            </p>
            <h2 className="mt-4 text-4xl font-bold leading-[1.08] tracking-tight sm:text-5xl">
              Les votes de l’Assemblée,
              <br />
              <span className="text-ink-3">sur pièces.</span>
            </h2>
            <p className="mt-6 max-w-xl text-base leading-relaxed text-ink-2">
              Les votes des députés sont publics et horodatés, mais dispersés
              dans des jeux de données que personne ne consulte. Ce site les
              rassemble : voici les textes, voici comment chaque groupe a voté,
              voici la source officielle.
            </p>
            <div className="mt-8 flex flex-wrap gap-3">
              <Link
                to="/scrutins"
                className="rounded-lg bg-ink px-4 py-2.5 text-sm font-medium text-surface transition-opacity hover:opacity-90"
              >
                Parcourir les scrutins
              </Link>
              <Link
                to="/comprendre"
                className="rounded-lg border border-line-strong px-4 py-2.5 text-sm font-medium text-ink-1 transition-colors hover:border-line-stronger"
              >
                Comment lire une page de vote
              </Link>
            </div>
          </div>

          <div className="rounded-2xl border border-line bg-raised p-6">
            {dernier ? (
              <>
                <div className="flex items-baseline justify-between gap-3">
                  <p
                    id="hemicycle-titre"
                    className="text-xs font-semibold uppercase tracking-[0.14em] text-ink-4"
                  >
                    Dernier scrutin publié
                  </p>
                  <span className="text-xs text-ink-4">
                    {formatDate(dernier.date)}
                  </span>
                </div>

                {votesNominaux ? (
                  <>
                    <Hemicycle
                      votes={votesNominaux}
                      labelledBy="hemicycle-titre"
                    />

                    <div className="flex flex-wrap justify-center gap-x-5 gap-y-1.5 text-xs tabular-nums">
                      {positions(tallyDesVotes(votesNominaux)).map((bloc) => (
                        <span
                          key={bloc.cle}
                          className="flex items-center gap-1.5"
                        >
                          <span
                            className={`h-2 w-2 rounded-full ${bloc.fond}`}
                            aria-hidden
                          />
                          <span className={bloc.texte}>
                            {formatNombre(bloc.total)}
                          </span>
                          <span className="text-ink-4">
                            {bloc.libelleAccorde}
                          </span>
                        </span>
                      ))}
                    </div>
                  </>
                ) : (
                  <div className="flex h-56 items-center justify-center text-sm text-ink-4">
                    {detail.isError
                      ? 'Positions nominales indisponibles.'
                      : 'Chargement des positions…'}
                  </div>
                )}

                <p className="mt-5 line-clamp-2 text-sm leading-snug text-ink-1">
                  {dernier.subject}
                </p>

                <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
                  <Link
                    to={`/scrutins/${dernier.uid}`}
                    className="text-link-ink hover:underline"
                  >
                    Voir le détail du vote
                  </Link>
                  <a
                    href={dernier.official_url}
                    target="_blank"
                    rel="noreferrer"
                    className="text-ink-4 underline decoration-line-strong underline-offset-2 hover:text-ink-2"
                  >
                    Source officielle
                  </a>
                </div>

                <p className="mt-4 border-t border-line pt-3 text-xs leading-relaxed text-ink-4">
                  Chaque siège porte le vote du député qui l’occupe, à sa place
                  réelle sur le{' '}
                  <a
                    href="https://www.assemblee-nationale.fr/dyn/vos-deputes/hemicycle"
                    target="_blank"
                    rel="noreferrer"
                    className="underline decoration-line-strong underline-offset-2 hover:text-ink-2"
                  >
                    plan de l’hémicycle
                  </a>{' '}
                  publié par l’Assemblée. En gris, les sièges dont aucune
                  position n’a été enregistrée sur ce scrutin.
                </p>
              </>
            ) : (
              <div className="flex h-64 items-center justify-center text-sm text-ink-4">
                {scrutins.isError
                  ? 'Dernier scrutin indisponible.'
                  : 'Chargement du dernier scrutin…'}
              </div>
            )}
          </div>
        </div>
      </Section>

      {/* ------------------------------------------------------------- Principe */}
      <Section className="border-y border-line bg-raised py-16">
        <TitreSection
          surtitre="Le principe"
          titre="Trois étapes, et une seule où un jugement intervient"
        />

        <ol className="mt-8 grid gap-6 md:grid-cols-3">
          {PRINCIPE.map((etape, index) => (
            <li key={etape.titre} className="border-t border-line-strong pt-4">
              <span className="text-xs font-semibold tabular-nums text-ink-4">
                {String(index + 1).padStart(2, '0')}
              </span>
              <h3 className="mt-2 text-base font-semibold">{etape.titre}</h3>
              <p className="mt-2 text-sm leading-relaxed text-ink-3">
                {etape.texte}
              </p>
            </li>
          ))}
        </ol>

        <p className="mt-8 text-sm text-ink-3">
          La méthode de rattachement aux thèmes est publiée :{' '}
          <Link
            to="/themes/methode"
            className="text-link-ink underline underline-offset-2 hover:no-underline"
          >
            comment un texte reçoit sa famille
          </Link>
          .
        </p>
      </Section>

      {/* ------------------------------------------------ Derniers scrutins */}
      <Section className="py-16">
        <div className="flex flex-wrap items-end justify-between gap-4">
          <TitreSection
            surtitre="Publié récemment"
            titre={`Les ${DERNIERS} derniers scrutins`}
          />
          <Link
            to="/scrutins"
            className="text-sm text-link-ink hover:underline"
          >
            Tous les scrutins →
          </Link>
        </div>

        <div className="mt-8 grid gap-4 md:grid-cols-3">
          {scrutins.data
            ? scrutins.data.scrutins.map((scrutin) => (
                <CarteScrutin key={scrutin.uid} scrutin={scrutin} />
              ))
            : Array.from({ length: DERNIERS }).map((_, index) => (
                <div
                  key={index}
                  className="h-56 animate-pulse rounded-xl border border-line bg-raised"
                />
              ))}
        </div>

        {scrutins.isError && (
          <p className="mt-4 text-sm text-against-ink">
            Scrutins indisponibles :{' '}
            {scrutins.error instanceof Error
              ? scrutins.error.message
              : 'erreur inconnue'}
          </p>
        )}

        {scrutins.data && (
          <p className="mt-4 rounded-lg border border-line bg-raised px-3 py-2 text-xs leading-relaxed text-ink-4">
            {scrutins.data.coverage_note}
          </p>
        )}
      </Section>

      {/* ------------------------------------------------------ Anti-promesses */}
      <Section className="border-y border-line bg-raised py-16">
        <div className="grid gap-10 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
          <TitreSection
            surtitre="Anti-promesse"
            titre="Ce que ce site ne fait pas"
            chapeau="Le test tient en une question : une personne de droite et une personne de gauche trouveraient-elles la page juste ?"
          />
          <ul className="space-y-3">
            {ANTI_PROMESSES.map((regle) => (
              <li
                key={regle}
                className="border-l-2 border-line-strong pl-4 text-sm leading-relaxed text-ink-2"
              >
                {regle}
              </li>
            ))}
          </ul>
        </div>
      </Section>

      {/* ---------------------------------------------------------- Sources */}
      <Section>
        <div className="rounded-xl border border-line bg-raised p-6">
          <h3 className="text-sm font-semibold uppercase tracking-[0.14em] text-ink-4">
            Sources
          </h3>
          <p className="mt-3 max-w-3xl text-sm leading-relaxed text-ink-3">
            Les données proviennent de l’open data de l’Assemblée nationale,
            sous Licence Ouverte : scrutins et positions nominales, référentiel
            des députés et des groupes avec leurs appartenances datées, dossiers
            législatifs. Le Sénat est hors périmètre.
          </p>
          <a
            href="https://data.assemblee-nationale.fr"
            target="_blank"
            rel="noreferrer"
            className="mt-3 inline-block text-sm text-link-ink hover:underline"
          >
            data.assemblee-nationale.fr
          </a>
        </div>
      </Section>
    </div>
  )
}
