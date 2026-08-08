import { Outlet, Link, NavLink } from 'react-router'
import Logo from './components/Logo'
import ThemeToggle from './components/ThemeToggle'

/*
 * Quatre entrées, pas sept. « Accueil » est déjà porté par le logo ; les
 * scrutins s'atteignent depuis les dossiers, dont ils sont le détail ; les
 * fiches de groupes depuis la page qui compare leurs votes, où l'on va
 * justement chercher qui est qui. Une barre qui liste les tables du modèle
 * fait porter au lecteur le travail de savoir laquelle le concerne.
 */
const NAV = [
  { to: '/dossiers', label: 'Dossiers', end: false },
  {
    to: '/votes-par-groupe',
    label: 'Comparer les votes des groupes',
    end: false,
  },
  { to: '/themes', label: 'Thèmes', end: false },
  { to: '/comprendre', label: 'Comprendre', end: false },
]

export default function App() {
  return (
    <div className="flex min-h-screen flex-col bg-canvas text-ink">
      {/*
        En-tête collant : sur des listes de plusieurs milliers de lignes, une
        navigation qu'il faut remonter chercher n'est pas une navigation.
      */}
      <header className="sticky top-0 z-20 border-b border-line bg-surface/85 backdrop-blur-md">
        {/* Sur petit écran le titre et la navigation s'empilent : côte à côte,
            la nav se replie sur trois lignes contre le logo. */}
        <div className="mx-auto flex max-w-7xl flex-col gap-2 px-6 py-2.5 sm:flex-row sm:items-center sm:justify-between sm:gap-8">
          <Link
            to="/"
            className="flex items-center gap-2 transition-opacity hover:opacity-70"
          >
            <Logo className="h-9 w-auto shrink-0" />
            <h1 className="text-[15px] font-semibold tracking-tight">
              hémicycle
              <span className="text-accent">.data</span>
            </h1>
          </Link>

          <nav className="-mx-1 flex flex-wrap items-center gap-0.5 sm:justify-end">
            {NAV.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  `rounded-lg px-2.5 py-1.5 text-[13px] font-medium whitespace-nowrap transition-colors ${
                    isActive
                      ? 'bg-surface-soft text-ink'
                      : 'text-ink-soft hover:bg-surface-soft hover:text-ink'
                  }`
                }
              >
                {item.label}
              </NavLink>
            ))}
            <span className="mx-1 h-4 w-px bg-line" aria-hidden />
            <ThemeToggle />
          </nav>
        </div>
      </header>

      <main className="mx-auto w-full max-w-7xl flex-1 px-6 py-8">
        <Outlet />
      </main>

      <footer className="mt-12 border-t border-line px-6 py-6">
        <div className="mx-auto flex max-w-7xl flex-wrap items-baseline gap-x-5 gap-y-2 text-xs">
          <span className="font-medium text-ink-soft">
            Transparence des votes parlementaires
          </span>
          <span className="text-ink-faint">
            Données publiques de l'Assemblée nationale, reprises sans sélection.
          </span>
          <NavLink
            to="/selection"
            className={({ isActive }) =>
              `ml-auto ${isActive ? 'text-ink-soft underline underline-offset-4' : 'text-ink-faint hover:text-ink-soft'}`
            }
          >
            Sélection des dossiers
          </NavLink>
        </div>
      </footer>
    </div>
  )
}
