import { Outlet, Link, NavLink, useLocation } from 'react-router'
import ThemeToggle from './components/ThemeToggle'

const NAV = [
  { to: '/', label: 'Accueil', end: true },
  { to: '/dossiers', label: 'Dossiers', end: false },
  { to: '/groupes', label: 'Groupes', end: false },
  { to: '/votes-par-groupe', label: 'Votes par groupe', end: false },
  { to: '/themes', label: 'Thèmes', end: false },
  { to: '/comprendre', label: 'Comprendre', end: false },
]

const NAV_SECONDAIRE = [
  { to: '/scrutins', label: 'Scrutins' },
  { to: '/selection', label: 'Sélection des dossiers' },
]

export default function App() {
  /**
   * L'accueil est la seule page composée en pleine largeur : hero et grille de
   * cartes. Les pages de consultation gardent la colonne étroite, réglée pour
   * la lecture de listes de votes.
   */
  const pleineLargeur = useLocation().pathname === '/'

  return (
    <div className="min-h-screen flex flex-col bg-surface text-ink">
      <header className="border-b border-line px-6 py-4">
        <div className="max-w-6xl mx-auto flex flex-wrap items-end justify-between gap-4">
          <Link to="/" className="hover:opacity-80">
            <h1 className="text-2xl font-bold tracking-tight">hémicycle.data</h1>
            <p className="text-sm text-ink-4">
              Transparence des votes parlementaires
            </p>
          </Link>
          <div className="flex items-center gap-2">
            <nav className="flex flex-wrap justify-end gap-1">
              {NAV.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  end={item.end}
                  className={({ isActive }) =>
                    `whitespace-nowrap px-3 py-1 rounded text-sm ${
                      isActive
                        ? 'bg-sunken text-ink'
                        : 'text-ink-3 hover:text-ink-1'
                    }`
                  }
                >
                  {item.label}
                </NavLink>
              ))}
            </nav>
            <ThemeToggle />
          </div>
        </div>
      </header>

      <main
        className={`w-full flex-1 ${
          pleineLargeur ? '' : 'max-w-3xl mx-auto px-6 py-8'
        }`}
      >
        <Outlet />
      </main>

      <footer className="border-t border-line px-6 py-6 mt-8">
        <div className="max-w-6xl mx-auto">
          <nav className="flex flex-wrap gap-x-4 gap-y-2 text-sm">
            {NAV_SECONDAIRE.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  isActive
                    ? 'text-ink-1 underline underline-offset-4'
                    : 'text-ink-4 hover:text-ink-2'
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </div>
      </footer>
    </div>
  )
}
