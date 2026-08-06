import { useEffect, useRef, useState } from 'react'
import { Link, useLocation, useNavigate, useSearchParams } from 'react-router'
import {
  QUESTIONS,
  SECTIONS,
  type Bloc,
  type Niveau,
  type Question,
  type Section,
} from '../content/comprendre'

const NIVEAUX: { valeur: Niveau; label: string; aide: string }[] = [
  {
    valeur: 'debutant',
    label: 'Débutant',
    aide: 'Le vocabulaire et de quoi lire une page de vote sans se tromper.',
  },
  {
    valeur: 'detaille',
    label: 'Détaillé',
    aide: 'Suppose le niveau débutant acquis, et ne le répète pas : procédure, cas particuliers, limites des données.',
  },
]

const CLE_NIVEAU = 'comprendre.niveau'

/**
 * L'URL fait foi : un lien partagé doit ouvrir le niveau qu'il désigne. Sans
 * paramètre, on reprend le dernier choix du lecteur, pour que les renvois
 * depuis les pages de vote n'annulent pas son réglage.
 */
function niveauInitial(brut: string | null): Niveau {
  if (brut === 'detaille' || brut === 'debutant') return brut
  return localStorage.getItem(CLE_NIVEAU) === 'detaille' ? 'detaille' : 'debutant'
}

/** Niveaux disjoints : le détaillé ne redit pas ce que le débutant a déjà dit. */
function visible(question: Question, niveau: Niveau): boolean {
  return question.niveau === niveau
}

/** Seul `<strong>` est admis dans le contenu — pas de HTML injecté. */
function RichText({ texte }: { texte: string }) {
  const morceaux = texte.split(/<strong>(.*?)<\/strong>/g)
  return (
    <>
      {morceaux.map((morceau, index) =>
        index % 2 === 1 ? (
          <strong key={index} className="font-semibold text-gray-200">
            {morceau}
          </strong>
        ) : (
          morceau
        ),
      )}
    </>
  )
}

function BlocView({ bloc }: { bloc: Bloc }) {
  if (bloc.kind === 'p') {
    return (
      <p className="text-sm leading-relaxed text-gray-400">
        <RichText texte={bloc.texte} />
      </p>
    )
  }

  if (bloc.kind === 'ul') {
    return (
      <div>
        {bloc.intro && <p className="text-sm text-gray-400">{bloc.intro}</p>}
        <ul className="mt-2 space-y-2">
          {bloc.items.map((item, index) => (
            <li
              key={index}
              className="border-l-2 border-gray-800 pl-3 text-sm leading-relaxed text-gray-400"
            >
              <RichText texte={item} />
            </li>
          ))}
        </ul>
      </div>
    )
  }

  return (
    <p className="text-sm leading-relaxed text-gray-400">
      <a
        href={bloc.lien}
        target="_blank"
        rel="noreferrer"
        className="text-gray-200 underline decoration-gray-600 underline-offset-2 hover:decoration-gray-300"
      >
        {bloc.libelle}
      </a>
      {bloc.precision && <> — {bloc.precision}</>}
    </p>
  )
}

function QuestionView({
  question,
  ouverte,
  onBascule,
}: {
  question: Question
  ouverte: boolean
  onBascule: (ouvert: boolean) => void
}) {
  return (
    <details
      id={question.id}
      open={ouverte}
      onToggle={(event) => onBascule(event.currentTarget.open)}
      className="scroll-mt-6 border-b border-gray-800"
    >
      <summary className="cursor-pointer list-none py-3 text-sm text-gray-200 marker:content-none hover:text-white">
        <span className="mr-2 inline-block w-3 text-gray-600">
          {ouverte ? '−' : '+'}
        </span>
        {question.question}
      </summary>
      <div className="space-y-3 pb-4 pl-5">
        {question.reponse.map((bloc, index) => (
          <BlocView key={index} bloc={bloc} />
        ))}
      </div>
    </details>
  )
}

function SectionView({
  section,
  niveau,
  ouvertes,
  onBascule,
}: {
  section: Section
  niveau: Niveau
  ouvertes: Set<string>
  onBascule: (id: string, ouvert: boolean) => void
}) {
  const questions = section.questions.filter((q) => visible(q, niveau))
  if (questions.length === 0) return null
  return (
    <section id={section.id} className="scroll-mt-6">
      <h3 className="mb-1 text-xs font-semibold uppercase tracking-wide text-gray-500">
        {section.titre}
      </h3>
      <div className="border-t border-gray-800">
        {questions.map((question) => (
          <QuestionView
            key={question.id}
            question={question}
            ouverte={ouvertes.has(question.id)}
            onBascule={(ouvert) => onBascule(question.id, ouvert)}
          />
        ))}
      </div>
    </section>
  )
}

/**
 * Guide de lecture, en questions — PROJECT.md §2, §3, §7, §9.
 *
 * Page entièrement statique : aucun chiffre n'y est affiché, donc aucune
 * requête. Les volumétries vivent sur les pages qui les servent depuis la base.
 */
export default function ComprendrePage() {
  const [params] = useSearchParams()
  const navigate = useNavigate()
  const { hash } = useLocation()
  const niveau = niveauInitial(params.get('niveau'))
  const [ouvertes, setOuvertes] = useState<Set<string>>(new Set())
  /** Dernière ancre effectivement rejointe : empêche de re-défiler à chaque
   * changement de niveau alors que le lecteur est déjà arrivé. */
  const ancreTraitee = useRef<string | null>(null)

  /** Change de niveau sans perdre l'ancre : la réécrire sans le hash
   * ramènerait le lecteur en haut de page. */
  function appliquerNiveau(valeur: Niveau) {
    localStorage.setItem(CLE_NIVEAU, valeur)
    const suivant = new URLSearchParams(params)
    suivant.set('niveau', valeur)
    navigate({ search: `?${suivant}`, hash }, { replace: true })
  }

  /**
   * Un renvoi venu d'une page de vote arrive par navigation cliente : la
   * question n'existe pas encore quand le navigateur traite l'ancre, et rien ne
   * défile. On la déplie et on rejoue le saut une fois le contenu rendu.
   *
   * Les deux niveaux étant disjoints, l'ancre peut viser une question absente
   * du niveau courant, dans un sens comme dans l'autre. On bascule alors vers
   * le niveau qui la porte et on laisse l'effet repasser : renvoyer vers une
   * ancre invisible serait un lien mort.
   */
  useEffect(() => {
    if (!hash) return
    const id = hash.slice(1)
    if (ancreTraitee.current === id) return

    const question = QUESTIONS.find((q) => q.id === id)
    if (question && question.niveau !== niveau) {
      appliquerNiveau(question.niveau)
      return
    }
    if (question) setOuvertes((prev) => new Set(prev).add(id))

    const cible = document.getElementById(id)
    if (!cible) return
    ancreTraitee.current = id

    // Le dépliage de la question modifie encore la hauteur au commit suivant :
    // défiler tout de suite laisse la cible hors écran. On attend que la mise
    // en page soit stabilisée.
    const frame = requestAnimationFrame(() =>
      requestAnimationFrame(() => cible.scrollIntoView()),
    )
    return () => cancelAnimationFrame(frame)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [hash, niveau])

  function basculerQuestion(id: string, ouvert: boolean) {
    setOuvertes((prev) => {
      const suivant = new Set(prev)
      if (ouvert) suivant.add(id)
      else suivant.delete(id)
      return suivant
    })
  }

  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-xl font-bold">Comprendre</h2>
        <p className="mt-2 text-sm leading-relaxed text-gray-400">
          Ce que ce site affiche, ce que les mots recouvrent, et ce que les
          données ne permettent pas de dire.
        </p>
      </div>

      <div>
        <div role="tablist" aria-label="Niveau de lecture" className="flex gap-1">
          {NIVEAUX.map((item) => (
            <button
              key={item.valeur}
              type="button"
              role="tab"
              aria-selected={niveau === item.valeur}
              onClick={() => appliquerNiveau(item.valeur)}
              className={`rounded px-3 py-1 text-sm ${
                niveau === item.valeur
                  ? 'bg-gray-800 text-white'
                  : 'text-gray-400 hover:text-gray-200'
              }`}
            >
              {item.label}
            </button>
          ))}
        </div>
        <p className="mt-2 text-xs text-gray-500">
          {NIVEAUX.find((item) => item.valeur === niveau)!.aide}
        </p>
      </div>

      <div className="space-y-8">
        {SECTIONS.map((section) => (
          <SectionView
            key={section.id}
            section={section}
            niveau={niveau}
            ouvertes={ouvertes}
            onBascule={basculerQuestion}
          />
        ))}
      </div>

      {niveau === 'debutant' && (
        <p className="border-t border-gray-800 pt-4 text-sm text-gray-500">
          Ces réponses vous paraissent acquises ?{' '}
          <button
            type="button"
            onClick={() => appliquerNiveau('detaille')}
            className="text-gray-300 underline decoration-gray-600 underline-offset-2 hover:decoration-gray-300"
          >
            Passer au niveau détaillé
          </button>{' '}
          — procédure, cas particuliers, limites des données.
        </p>
      )}

      <p className="border-t border-gray-800 pt-4 text-sm text-gray-500">
        La méthode de rattachement aux thèmes est publiée à part :{' '}
        <Link
          to="/themes/methode"
          className="text-gray-300 underline decoration-gray-600 underline-offset-2 hover:decoration-gray-300"
        >
          méthode de thématisation
        </Link>
        .
      </p>
    </div>
  )
}
