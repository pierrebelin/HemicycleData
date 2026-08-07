import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import Hemicycle from '../components/Hemicycle'
import ScrutinList, { CoverageNote } from '../components/ScrutinList'
import { siegesDesGroupes, tallyDesVotes } from '../components/sieges'
import { Card, ErrorPanel, SectionTitle, TallyLine } from '../components/ui'
import type { ScrutinDetailDto, ScrutinListResponse } from '../types/scrutins'
import { formatDate } from '../types/scrutins'

const DERNIERS = 3

/**
 * Ce que le site refuse de faire, énoncé à l'accueil plutôt qu'enfoui dans une
 * page annexe : c'est la contrepartie qui rend la promesse vérifiable
 * (README.md §6).
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

  /*
   * Le résumé de liste ne porte que les totaux. Placer chaque vote sur son
   * siège demande les positions nominales, donc le détail du scrutin — qui
   * fournit aussi le numéro de législature affiché en surtitre.
   */
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
    <div className="space-y-12">
      {/* ---------------------------------------------------------------- Hero */}
      <section className="grid items-center gap-8 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.05fr)]">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint">
            Assemblée nationale
            {detail.data && <> · {detail.data.legislature}ᵉ législature</>}
          </p>
          <h2 className="mt-3 text-4xl font-semibold leading-[1.1] tracking-tight sm:text-5xl">
            Les votes de l’Assemblée,
            <br />
            <span className="text-ink-soft">sur pièces.</span>
          </h2>
          <p className="mt-5 max-w-xl text-base leading-relaxed text-ink-soft">
            Les votes des députés sont publics et horodatés, mais dispersés dans
            des jeux de données que personne ne consulte. Ce site les rassemble :
            voici les textes, voici comment chaque groupe a voté, voici la source
            officielle.
          </p>
          <div className="mt-7 flex flex-wrap gap-3">
            <Link
              to="/scrutins"
              className="rounded-lg bg-ink px-4 py-2.5 text-sm font-medium text-surface transition-opacity hover:opacity-85"
            >
              Parcourir les scrutins
            </Link>
            <Link
              to="/comprendre"
              className="rounded-lg border border-line bg-surface px-4 py-2.5 text-sm font-medium text-ink-soft shadow-card transition-colors hover:bg-surface-soft hover:text-ink"
            >
              Comment lire une page de vote
            </Link>
          </div>
        </div>

        <Card className="px-5 py-4">
          {dernier && votesNominaux ? (
            <>
              <div className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
                <h3
                  id="hemicycle-accueil"
                  className="text-xs font-semibold uppercase tracking-[0.06em] text-ink-faint"
                >
                  Dernier scrutin publié
                </h3>
                <span className="text-xs text-ink-faint">
                  {formatDate(dernier.date)}
                </span>
              </div>

              <Hemicycle votes={votesNominaux} labelledBy="hemicycle-accueil" />

              <div className="flex justify-center">
                <TallyLine tally={tallyDesVotes(votesNominaux)} />
              </div>

              <p className="mt-4 line-clamp-2 text-sm leading-snug text-ink">
                {dernier.subject}
              </p>

              <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs">
                <Link
                  to={`/scrutins/${dernier.uid}`}
                  className="font-medium text-accent hover:underline"
                >
                  Voir le détail du vote →
                </Link>
                <a
                  href={dernier.official_url}
                  target="_blank"
                  rel="noreferrer"
                  className="text-ink-faint hover:text-ink-soft hover:underline"
                >
                  Source officielle ↗
                </a>
              </div>

              <p className="mt-4 border-t border-line pt-3 text-xs leading-relaxed text-ink-faint">
                Chaque siège porte le vote du député qui l’occupe, à sa place
                réelle sur le{' '}
                <a
                  href="https://www.assemblee-nationale.fr/dyn/vos-deputes/hemicycle"
                  target="_blank"
                  rel="noreferrer"
                  className="text-accent hover:underline"
                >
                  plan de l’hémicycle
                </a>{' '}
                publié par l’Assemblée. En gris, les sièges dont aucune position
                n’a été enregistrée sur ce scrutin.
              </p>
            </>
          ) : (
            <p className="flex h-64 items-center justify-center text-sm text-ink-faint">
              {scrutins.isError || detail.isError
                ? 'Dernier scrutin indisponible.'
                : 'Chargement du dernier scrutin…'}
            </p>
          )}
        </Card>
      </section>

      {/* ------------------------------------------------------------- Principe */}
      <section>
        <SectionTitle>Le principe</SectionTitle>
        <h3 className="mb-5 max-w-2xl text-2xl font-semibold tracking-tight">
          Trois étapes, et une seule où un jugement intervient
        </h3>

        <ol className="grid gap-4 md:grid-cols-3">
          {PRINCIPE.map((etape, index) => (
            <li key={etape.titre}>
              <Card className="h-full px-5 py-4">
                <span className="text-xs font-semibold text-ink-faint">
                  {String(index + 1).padStart(2, '0')}
                </span>
                <h4 className="mt-1.5 text-sm font-semibold">{etape.titre}</h4>
                <p className="mt-2 text-sm leading-relaxed text-ink-soft">
                  {etape.texte}
                </p>
              </Card>
            </li>
          ))}
        </ol>

        <p className="mt-4 text-sm text-ink-soft">
          La méthode de rattachement aux thèmes est publiée :{' '}
          <Link to="/themes/methode" className="text-accent hover:underline">
            comment un texte reçoit sa famille
          </Link>
          .
        </p>
      </section>

      {/* ------------------------------------------------ Derniers scrutins */}
      <section>
        <div className="mb-2 flex flex-wrap items-baseline justify-between gap-x-6 gap-y-1">
          <SectionTitle>Publié récemment</SectionTitle>
          <Link
            to="/scrutins"
            className="text-xs font-medium text-accent hover:underline"
          >
            Tous les scrutins →
          </Link>
        </div>

        {scrutins.isError && <ErrorPanel error={scrutins.error} />}
        {scrutins.data && (
          <div className="space-y-3">
            <ScrutinList scrutins={scrutins.data.scrutins} />
            <CoverageNote note={scrutins.data.coverage_note} />
          </div>
        )}
      </section>

      {/* ------------------------------------------------------ Anti-promesses */}
      <section className="grid gap-6 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
        <div>
          <SectionTitle>Anti-promesse</SectionTitle>
          <h3 className="text-2xl font-semibold tracking-tight">
            Ce que ce site ne fait pas
          </h3>
          <p className="mt-2 max-w-md text-sm leading-relaxed text-ink-soft">
            Le test tient en une question : une personne de droite et une
            personne de gauche trouveraient-elles la page juste ?
          </p>
        </div>
        <ul className="space-y-2.5">
          {ANTI_PROMESSES.map((regle) => (
            <li
              key={regle}
              className="border-l-2 border-line-strong pl-4 text-sm leading-relaxed text-ink-soft"
            >
              {regle}
            </li>
          ))}
        </ul>
      </section>

      {/* ---------------------------------------------------------- Sources */}
      <section>
        <SectionTitle>Sources</SectionTitle>
        <Card className="px-5 py-4">
          <p className="max-w-4xl text-sm leading-relaxed text-ink-soft">
            Les données proviennent de l’open data de l’Assemblée nationale, sous
            Licence Ouverte : scrutins et positions nominales, référentiel des
            députés et des groupes avec leurs appartenances datées, dossiers
            législatifs. Le Sénat est hors périmètre.
          </p>
          <a
            href="https://data.assemblee-nationale.fr"
            target="_blank"
            rel="noreferrer"
            className="mt-2 inline-block text-sm font-medium text-accent hover:underline"
          >
            data.assemblee-nationale.fr ↗
          </a>
        </Card>
      </section>
    </div>
  )
}
