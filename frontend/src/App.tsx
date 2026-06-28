import { Outlet, Link } from 'react-router'

export default function App() {
  return (
    <div className="min-h-screen bg-gray-950 text-white">
      <header className="border-b border-gray-800 px-6 py-4">
        <div className="max-w-3xl mx-auto">
          <Link to="/" className="hover:opacity-80">
            <h1 className="text-2xl font-bold">hémicycle.data</h1>
            <p className="text-sm text-gray-500">Veille parlementaire</p>
          </Link>
        </div>
      </header>
      <main className="max-w-3xl mx-auto px-6 py-8">
        <Outlet />
      </main>
    </div>
  )
}
