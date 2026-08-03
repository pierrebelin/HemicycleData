import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import { MethodNote } from '../components/ThemeBadges'
import type { FamiliesResponse, MethodResponse } from '../types/themes'

export default function ThemeListPage() {
  const families = useQuery({
    queryKey: ['themes'],
    queryFn: (): Promise<FamiliesResponse> =>
      fetch('/api/themes').then((res) => {
        if (!res.ok) throw new Error('Chargement des thèmes impossible')
        return res.json()
      }),
  })

  const method = useQuery({
    queryKey: ['themes', 'method'],
    queryFn: (): Promise<MethodResponse> =>
      fetch('/api/themes/method').then((res) => {
        if (!res.ok) throw new Error('Chargement de la méthode impossible')
        return res.json()
      }),
  })

  const coverage = new Map(
    method.data?.families.map((f) => [f.code, f]) ?? [],
  )

  if (families.isLoading) {
    return <p className="text-gray-500">Chargement…</p>
  }
  if (families.error) {
    return <p className="text-red-400">{(families.error as Error).message}</p>
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-xl font-bold">Thèmes</h2>
        <p className="mt-1 text-sm text-gray-400">
          Les votes de la législature, regroupés par famille thématique. Un
          texte peut appartenir à {families.data?.max_families_per_text}{' '}
          familles au plus.
        </p>
      </div>

      <MethodNote note={families.data!.method_note} />

      <ul className="space-y-3">
        {families.data!.families.map((family) => {
          const stats = coverage.get(family.code)
          return (
            <li
              key={family.code}
              className="rounded border border-gray-800 bg-gray-900/40 p-4"
            >
              <Link
                to={`/themes/${family.code}`}
                className="font-medium hover:underline"
              >
                {family.label}
              </Link>
              {stats && (
                <p className="mt-1 text-xs text-gray-500">
                  {stats.text_count} texte{stats.text_count > 1 ? 's' : ''} ·{' '}
                  {stats.scrutin_count} scrutin
                  {stats.scrutin_count > 1 ? 's' : ''} ·{' '}
                  {stats.arbitrated_text_count} arbitré
                  {stats.arbitrated_text_count > 1 ? 's' : ''}
                </p>
              )}
              <p className="mt-2 text-sm text-gray-400">{family.scope}</p>
            </li>
          )
        })}
      </ul>

      {method.data && (
        <div className="rounded border border-gray-800 bg-gray-900/40 p-4">
          <Link
            to="/themes/non-rattaches"
            className="font-medium hover:underline"
          >
            Textes non rattachés
          </Link>
          <p className="mt-1 text-xs text-gray-500">
            {method.data.texts_total - method.data.texts_assigned} texte
            {method.data.texts_total - method.data.texts_assigned > 1
              ? 's'
              : ''}{' '}
            sans famille, sur {method.data.texts_total}. Ils restent
            consultables : rien n'est retiré du site.
          </p>
        </div>
      )}
    </div>
  )
}
