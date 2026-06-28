import { useQuery } from '@tanstack/react-query'

function App() {
  const { data, isLoading, isError } = useQuery({
    queryKey: ['health'],
    queryFn: () => fetch('/api/health').then((res) => res.json()),
  })

  return (
    <div className="min-h-screen bg-gray-950 text-white flex items-center justify-center">
      <div className="text-center space-y-6">
        <h1 className="text-4xl font-bold">HemicycleData</h1>
        <p className="text-gray-400">Générateur de posts Instagram</p>
        <div className="mt-8 p-4 rounded-lg bg-gray-900 border border-gray-800">
          <p className="text-sm text-gray-500 mb-2">API Health</p>
          {isLoading && <p className="text-yellow-400">Connexion...</p>}
          {isError && <p className="text-red-400">Backend injoignable</p>}
          {data && (
            <div className="space-y-1">
              <p className="text-green-400">Status: {data.status}</p>
              <p className={data.database === 'connected' ? 'text-green-400' : 'text-red-400'}>
                Database: {data.database}
              </p>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
