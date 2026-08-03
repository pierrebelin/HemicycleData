import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import TextList from '../components/TextList'
import type { TextListResponse } from '../types/themes'

const PAGE_SIZE = 50

/** CU-05 — Un objet non rattaché reste consultable (RM-01). */
export default function UnassignedTextsPage() {
  const [offset, setOffset] = useState(0)

  const texts = useQuery({
    queryKey: ['themes', 'unassigned', offset],
    queryFn: (): Promise<TextListResponse> =>
      fetch(`/api/themes/unassigned?limit=${PAGE_SIZE}&offset=${offset}`).then(
        (res) => {
          if (!res.ok) throw new Error('Chargement impossible')
          return res.json()
        },
      ),
  })

  if (texts.isLoading) return <p className="text-gray-500">Chargement…</p>
  if (texts.error)
    return <p className="text-red-400">{(texts.error as Error).message}</p>

  const data = texts.data!

  return (
    <div className="space-y-6">
      <div>
        <Link to="/themes" className="text-sm text-gray-500 hover:text-gray-300">
          ← Tous les thèmes
        </Link>
        <h2 className="mt-2 text-xl font-bold">Textes non rattachés</h2>
        <p className="mt-1 text-sm text-gray-400">
          Ces textes n'ont aucune famille. Ils restent consultables et leurs
          votes sont exposés comme les autres : le site ne retire rien faute de
          thème.
        </p>
        <p className="mt-2 text-xs text-gray-500">
          {data.total} texte{data.total > 1 ? 's' : ''}
        </p>
      </div>

      <TextList texts={data.items} showFamilies={false} />

      <div className="flex items-center justify-between">
        <button
          type="button"
          disabled={offset === 0}
          onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
          className="rounded border border-gray-700 px-3 py-1 text-sm disabled:opacity-40"
        >
          Précédent
        </button>
        <span className="text-xs text-gray-500">
          {data.total === 0
            ? '0'
            : `${offset + 1} – ${Math.min(offset + PAGE_SIZE, data.total)} sur ${data.total}`}
        </span>
        <button
          type="button"
          disabled={offset + PAGE_SIZE >= data.total}
          onClick={() => setOffset(offset + PAGE_SIZE)}
          className="rounded border border-gray-700 px-3 py-1 text-sm disabled:opacity-40"
        >
          Suivant
        </button>
      </div>
    </div>
  )
}
