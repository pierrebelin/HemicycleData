import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'

interface DossierDto {
  uid: string
  titre: string
  procedure: string
  derniere_activite_date: string
  derniere_activite_libelle: string
}

interface RecentDossiersResponse {
  count: number
  dossiers: DossierDto[]
}

function App() {
  const [days, setDays] = useState(7)

  const {
    data,
    isLoading,
    isError,
    error,
  } = useQuery<RecentDossiersResponse>({
    queryKey: ['dossiers', days],
    queryFn: () =>
      fetch(`/api/dossiers?days=${days}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
  })

  return (
    <div className="min-h-screen bg-gray-950 text-white">
      <header className="border-b border-gray-800 px-6 py-4">
        <div className="max-w-3xl mx-auto">
          <h1 className="text-2xl font-bold">hémicycle.data</h1>
          <p className="text-sm text-gray-500">Veille parlementaire</p>
        </div>
      </header>

      <main className="max-w-3xl mx-auto px-6 py-8">
        <div className="flex items-center gap-4 mb-8">
          <h2 className="text-xl font-semibold">Dossiers actifs</h2>
          <div className="flex gap-1">
            {[7, 14, 30].map((d) => (
              <button
                key={d}
                onClick={() => setDays(d)}
                className={`px-3 py-1 rounded text-sm ${
                  days === d
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                }`}
              >
                {d}j
              </button>
            ))}
          </div>
          {data && (
            <span className="text-sm text-gray-500 ml-auto">
              {data.count} dossier{data.count > 1 ? 's' : ''}
            </span>
          )}
        </div>

        {isLoading && (
          <div className="text-center py-20">
            <p className="text-gray-400 animate-pulse">
              Téléchargement des données de l'Assemblée nationale…
            </p>
          </div>
        )}

        {isError && (
          <div className="bg-red-900/20 border border-red-800 rounded-lg p-4">
            <p className="text-red-400">
              Erreur : {error instanceof Error ? error.message : 'inconnue'}
            </p>
          </div>
        )}

        <div className="space-y-3">
          {data?.dossiers.map((d) => (
            <div
              key={d.uid}
              className="bg-gray-900 border border-gray-800 rounded-lg p-4"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <p className="text-white font-medium leading-snug">
                    {d.titre}
                  </p>
                  <p className="text-xs text-gray-500 mt-1">{d.procedure}</p>
                </div>
                <div className="flex flex-col items-end shrink-0 text-right">
                  <span className="text-xs text-gray-400">
                    {new Date(
                      d.derniere_activite_date + 'T00:00:00',
                    ).toLocaleDateString('fr-FR', {
                      day: 'numeric',
                      month: 'short',
                    })}
                  </span>
                  <span className="text-xs text-blue-400 mt-0.5">
                    {d.derniere_activite_libelle}
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      </main>
    </div>
  )
}

export default App
