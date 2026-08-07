import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link, useParams } from 'react-router'
import TextList from '../components/TextList'
import { MethodNote } from '../components/ThemeBadges'
import type { FamiliesResponse, TextListResponse } from '../types/themes'

const PAGE_SIZE = 50

export default function ThemeDetailPage() {
  const { code } = useParams<{ code: string }>()
  const [offset, setOffset] = useState(0)

  const families = useQuery({
    queryKey: ['themes'],
    queryFn: (): Promise<FamiliesResponse> =>
      fetch('/api/themes').then((res) => res.json()),
  })

  const texts = useQuery({
    queryKey: ['themes', code, offset],
    queryFn: (): Promise<TextListResponse> =>
      fetch(`/api/themes/${code}?limit=${PAGE_SIZE}&offset=${offset}`).then(
        (res) => {
          if (res.status === 404) throw new Error('Famille inconnue')
          if (!res.ok) throw new Error('Chargement impossible')
          return res.json()
        },
      ),
  })

  const family = families.data?.families.find((f) => f.code === code)

  if (texts.isLoading) return <p className="text-ink-4">Chargement…</p>
  if (texts.error)
    return <p className="text-against-ink">{(texts.error as Error).message}</p>

  const data = texts.data!

  return (
    <div className="space-y-6">
      <div>
        <Link to="/themes" className="text-sm text-ink-4 hover:text-ink-2">
          ← Tous les thèmes
        </Link>
        <h2 className="mt-2 text-xl font-bold">{family?.label ?? code}</h2>
        {family && (
          <p className="mt-1 text-sm text-ink-3">{family.scope}</p>
        )}
        <p className="mt-2 text-xs text-ink-4">
          {data.total} texte{data.total > 1 ? 's' : ''} rattaché
          {data.total > 1 ? 's' : ''}
        </p>
      </div>

      <MethodNote note={data.method_note} />

      <TextList texts={data.items} />

      <div className="flex items-center justify-between">
        <button
          type="button"
          disabled={offset === 0}
          onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
          className="rounded border border-line-strong px-3 py-1 text-sm disabled:opacity-40"
        >
          Précédent
        </button>
        <span className="text-xs text-ink-4">
          {data.total === 0
            ? '0'
            : `${offset + 1} – ${Math.min(offset + PAGE_SIZE, data.total)} sur ${data.total}`}
        </span>
        <button
          type="button"
          disabled={offset + PAGE_SIZE >= data.total}
          onClick={() => setOffset(offset + PAGE_SIZE)}
          className="rounded border border-line-strong px-3 py-1 text-sm disabled:opacity-40"
        >
          Suivant
        </button>
      </div>

      <p className="text-xs text-ink-4">
        <Link to="/themes/non-rattaches" className="underline hover:text-ink-2">
          Voir les textes non rattachés
        </Link>{' '}
        — aucun texte n'est retiré du site faute de thème.
      </p>
    </div>
  )
}
