import { Outlet, Link, NavLink } from 'react-router'

export default function App() {
  return (
    <div className="min-h-screen flex flex-col bg-gray-950 text-white">
      <header className="border-b border-gray-800 px-6 py-4">
        <div className="max-w-3xl mx-auto flex items-end justify-between gap-4">
          <Link to="/" className="hover:opacity-80">
            <h1 className="text-2xl font-bold">hémicycle.data</h1>
            <p className="text-sm text-gray-500">
              Transparence des votes parlementaires
            </p>
          </Link>
          <nav className="flex gap-1">
            {[
              { to: '/', label: 'Dossiers', end: true },
              { to: '/groupes', label: 'Votes par groupe', end: false },
              { to: '/themes', label: 'Thèmes', end: false },
            ].map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.end}
                className={({ isActive }) =>
                  `px-3 py-1 rounded text-sm ${
                    isActive
                      ? 'bg-gray-800 text-white'
                      : 'text-gray-400 hover:text-gray-200'
                  }`
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>
        </div>
      </header>
      <main className="max-w-3xl mx-auto w-full px-6 py-8 flex-1">
        <Outlet />
      </main>
      <footer className="border-t border-gray-800 px-6 py-6 mt-8">
        <div className="max-w-3xl mx-auto">
          <nav className="flex flex-wrap gap-x-4 gap-y-2 text-sm">
            {[
              { to: '/scrutins', label: 'Scrutins' },
              { to: '/selection', label: 'Sélection des dossiers' },
            ].map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  isActive
                    ? 'text-gray-200 underline underline-offset-4'
                    : 'text-gray-500 hover:text-gray-300'
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
