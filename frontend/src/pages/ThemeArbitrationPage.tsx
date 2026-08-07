import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { Link } from 'react-router'
import type {
  FamiliesResponse,
  TextListResponse,
  TextSummaryDto,
} from '../types/themes'

const TOKEN_KEY = 'hemicycle.adminToken'
const PAGE_SIZE = 50

/**
 * CU-03 — Écran d'arbitrage. Accès par jeton partagé : ce n'est pas une page
 * publique, mais elle ne cache rien que le site n'affiche déjà.
 */
export default function ThemeArbitrationPage() {
  const queryClient = useQueryClient()
  const [token, setToken] = useState(
    () => localStorage.getItem(TOKEN_KEY) ?? '',
  )
  const [source, setSource] = useState('unassigned')
  const [selected, setSelected] = useState<TextSummaryDto | null>(null)
  const [families, setFamilies] = useState<string[]>([])
  const [author, setAuthor] = useState('')
  const [motive, setMotive] = useState('')

  const referential = useQuery({
    queryKey: ['themes'],
    queryFn: (): Promise<FamiliesResponse> =>
      fetch('/api/themes').then((res) => res.json()),
  })

  const texts = useQuery({
    queryKey: ['arbitration', source],
    queryFn: (): Promise<TextListResponse> => {
      const url =
        source === 'unassigned'
          ? `/api/themes/unassigned?limit=${PAGE_SIZE}`
          : `/api/themes/${source}?limit=${PAGE_SIZE}`
      return fetch(url).then((res) => res.json())
    },
  })

  const arbitrate = useMutation({
    mutationFn: async () => {
      const response = await fetch('/api/themes/arbitrate', {
        method: 'POST',
        headers: {
          'content-type': 'application/json',
          'x-admin-token': token,
        },
        body: JSON.stringify({
          subject_kind: 'text',
          subject_id: selected!.key,
          families,
          author,
          motive: motive.trim() === '' ? null : motive,
        }),
      })
      if (!response.ok) throw new Error(await response.text())
      return response.json()
    },
    onSuccess: () => {
      localStorage.setItem(TOKEN_KEY, token)
      queryClient.invalidateQueries({ queryKey: ['arbitration'] })
      queryClient.invalidateQueries({ queryKey: ['themes'] })
      setSelected(null)
      setFamilies([])
      setMotive('')
    },
  })

  function select(text: TextSummaryDto) {
    setSelected(text)
    setFamilies(text.families.map((f) => f.code))
    setMotive('')
    arbitrate.reset()
  }

  const maxFamilies = referential.data?.max_families_per_text ?? 3
  const tooMany = families.length > maxFamilies
  const motiveRequired = families.length === 0 && motive.trim() === ''

  return (
    <div className="space-y-6">
      <div>
        <Link to="/themes" className="text-sm text-ink-4 hover:text-ink-2">
          ← Tous les thèmes
        </Link>
        <h2 className="mt-2 text-xl font-bold">Arbitrage des rattachements</h2>
        <p className="mt-1 text-sm text-ink-3">
          Retenir, corriger ou écarter les familles proposées. Chaque décision
          clôt le rattachement précédent et en ouvre un nouveau : l'historique
          est conservé.
        </p>
      </div>

      <div className="space-y-3 rounded border border-line bg-raised p-4">
        <label className="block text-sm">
          <span className="text-ink-3">Jeton d'accès</span>
          <input
            type="password"
            value={token}
            onChange={(e) => setToken(e.target.value)}
            className="mt-1 w-full rounded border border-line-strong bg-surface px-2 py-1 text-sm"
          />
        </label>
        <label className="block text-sm">
          <span className="text-ink-3">Auteur de la décision</span>
          <input
            value={author}
            onChange={(e) => setAuthor(e.target.value)}
            placeholder="votre nom"
            className="mt-1 w-full rounded border border-line-strong bg-surface px-2 py-1 text-sm"
          />
        </label>
        <label className="block text-sm">
          <span className="text-ink-3">Liste</span>
          <select
            value={source}
            onChange={(e) => {
              setSource(e.target.value)
              setSelected(null)
            }}
            className="mt-1 w-full rounded border border-line-strong bg-surface px-2 py-1 text-sm"
          >
            <option value="unassigned">Textes non rattachés</option>
            {referential.data?.families.map((family) => (
              <option key={family.code} value={family.code}>
                {family.label}
              </option>
            ))}
          </select>
        </label>
      </div>

      {selected && (
        <div className="space-y-3 rounded border border-line-strong bg-raised p-4">
          <p className="text-sm font-medium">{selected.label}</p>
          <p className="text-xs text-ink-4">
            {selected.scrutin_count} scrutin
            {selected.scrutin_count > 1 ? 's' : ''}
          </p>

          <div className="flex flex-wrap gap-2">
            {referential.data?.families.map((family) => {
              const checked = families.includes(family.code)
              return (
                <label
                  key={family.code}
                  className={`cursor-pointer rounded border px-2 py-1 text-xs ${
                    checked
                      ? 'border-line-stronger bg-sunken text-ink'
                      : 'border-line-strong text-ink-3'
                  }`}
                >
                  <input
                    type="checkbox"
                    className="mr-1.5"
                    checked={checked}
                    onChange={() =>
                      setFamilies(
                        checked
                          ? families.filter((c) => c !== family.code)
                          : [...families, family.code],
                      )
                    }
                  />
                  {family.label}
                </label>
              )
            })}
          </div>

          {tooMany && (
            <p className="text-xs text-abstain-ink">
              {maxFamilies} familles au plus.
            </p>
          )}

          <label className="block text-sm">
            <span className="text-ink-3">
              Motif {families.length === 0 ? '(obligatoire)' : '(facultatif)'}
            </span>
            <textarea
              value={motive}
              onChange={(e) => setMotive(e.target.value)}
              rows={2}
              className="mt-1 w-full rounded border border-line-strong bg-surface px-2 py-1 text-sm"
            />
          </label>

          <div className="flex items-center gap-3">
            <button
              type="button"
              disabled={
                arbitrate.isPending ||
                tooMany ||
                motiveRequired ||
                author.trim() === '' ||
                token.trim() === ''
              }
              onClick={() => arbitrate.mutate()}
              className="rounded bg-ink px-3 py-1 text-sm font-medium text-surface disabled:opacity-40"
            >
              {arbitrate.isPending ? 'Enregistrement…' : 'Enregistrer'}
            </button>
            <button
              type="button"
              onClick={() => setSelected(null)}
              className="text-sm text-ink-3 hover:text-ink-1"
            >
              Annuler
            </button>
          </div>

          {arbitrate.error && (
            <p className="text-xs text-against-ink">
              {(arbitrate.error as Error).message}
            </p>
          )}
        </div>
      )}

      {texts.isLoading ? (
        <p className="text-ink-4">Chargement…</p>
      ) : (
        <ul className="space-y-2">
          {texts.data?.items.map((text) => (
            <li
              key={text.key}
              className="flex items-start justify-between gap-3 rounded border border-line bg-raised p-3"
            >
              <div>
                <p className="text-sm">{text.label}</p>
                <p className="mt-1 text-xs text-ink-4">
                  {text.scrutin_count} scrutin
                  {text.scrutin_count > 1 ? 's' : ''}
                  {text.families.length > 0
                    ? ` · ${text.families.map((f) => `${f.label} (${f.origin_note})`).join(', ')}`
                    : ''}
                </p>
              </div>
              <button
                type="button"
                onClick={() => select(text)}
                className="shrink-0 rounded border border-line-strong px-2 py-1 text-xs hover:border-line-stronger"
              >
                Arbitrer
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
