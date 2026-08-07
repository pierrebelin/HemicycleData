import { useState } from 'react'
import { useParams, Link } from 'react-router'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import DossierScrutins from '../components/DossierScrutins'
import { OutcomePanel } from '../components/OutcomeBadge'
import type { OutcomeDto } from '../types/dossiers'

/** Actes visibles avant dépliage. Un dossier peut en porter des dizaines. */
const VISIBLE_ACTS = 3

interface ActeDto {
  date: string
  label: string
}

interface ScoreDto {
  progress: number
  magnitude: number
  momentum: number
  total: number
}

interface StageDto {
  label: string
  chamber: string
}

interface InitiatorGroupDto {
  uid: string
  abbrev: string
  label: string
  quality: string | null
}

interface InitiatorDto {
  full_name: string
  actor_uid: string | null
  role: string | null
  group: InitiatorGroupDto | null
  /** Date à laquelle le groupe a été lu. Toujours affichée avec le groupe. */
  reference_date: string | null
  official_url: string | null
}

interface DossierDetailDto {
  uid: string
  title: string
  procedure: string
  last_activity_date: string
  last_activity_label: string
  acts: ActeDto[]
  score: ScoreDto
  persisted: boolean
  current_stage: StageDto | null
  initiators: InitiatorDto[]
  committee: string | null
  curation_status?: string
  outcome: OutcomeDto
}

const curationLabels: Record<string, { label: string; classes: string }> = {
  new: { label: 'Nouveau', classes: 'bg-sunken border-line-strong text-ink-3' },
  selected: { label: 'Sélectionné', classes: 'bg-for-soft border-for-line text-for-ink' },
  dismissed: { label: 'Écarté', classes: 'bg-sunken border-line-strong text-ink-4' },
  published: { label: 'Publié', classes: 'bg-link-soft border-link-line text-link-ink' },
}

function ScoreBar({
  label,
  value,
  weight,
  color,
}: {
  label: string
  value: number
  weight: number
  color: string
}) {
  const pct = (value / 10) * 100
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-ink-2">
          {label}{' '}
          <span className="text-ink-4 text-xs">&times;{weight}</span>
        </span>
        <span className="text-ink-3">{value}/10</span>
      </div>
      <div className="h-2 bg-sunken rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full transition-all ${color}`}
          style={{ width: `${pct}%` }}
        />
      </div>
    </div>
  )
}

function scoreTotalColor(score: number) {
  if (score >= 60) return 'text-for-ink'
  if (score >= 30) return 'text-abstain-ink'
  return 'text-ink-3'
}

export default function DossierDetailPage() {
  const { uid } = useParams<{ uid: string }>()
  const queryClient = useQueryClient()
  const [allActsShown, setAllActsShown] = useState(false)

  const { data, isLoading, isError, error } = useQuery<DossierDetailDto>({
    queryKey: ['dossier', uid],
    queryFn: () =>
      fetch(`/api/dossiers/${uid}`).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
        return res.json()
      }),
    enabled: !!uid,
  })

  const curateMutation = useMutation({
    mutationFn: (status: string) =>
      fetch(`/api/dossiers/${uid}/curate`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ status }),
      }).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dossier', uid] })
      queryClient.invalidateQueries({ queryKey: ['suggestions'] })
    },
  })

  const saveMutation = useMutation({
    mutationFn: () =>
      fetch(`/api/dossiers/${uid}/save`, { method: 'POST' }).then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`)
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['dossier', uid] })
    },
  })

  if (isLoading) {
    return (
      <div className="text-center py-20">
        <p className="text-ink-3 animate-pulse">Chargement du dossier…</p>
      </div>
    )
  }

  if (isError) {
    return (
      <div>
        <Link
          to="/"
          className="text-link-ink hover:text-link-ink text-sm mb-4 inline-block"
        >
          ← Retour à la liste
        </Link>
        <div className="bg-against-softer border border-against-line rounded-lg p-4">
          <p className="text-against-ink">
            Erreur : {error instanceof Error ? error.message : 'inconnue'}
          </p>
        </div>
      </div>
    )
  }

  if (!data) return null

  return (
    <div>
      <Link
        to="/"
        className="text-link-ink hover:text-link-ink text-sm mb-6 inline-block"
      >
        ← Retour à la liste
      </Link>

      <div className="mb-8">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-2xl font-bold leading-snug mb-2">{data.title}</h2>
            <div className="flex flex-wrap items-center gap-3 text-sm">
              <span className="text-ink-3">{data.procedure}</span>
              {data.current_stage && (
                <>
                  <span className="text-ink-5">·</span>
                  <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-model-soft border border-model-line text-model-ink text-xs font-medium">
                    {data.current_stage.label}
                    {data.current_stage.chamber && (
                      <span className="text-model-ink">
                        — {data.current_stage.chamber}
                      </span>
                    )}
                  </span>
                </>
              )}
              <span className="text-ink-5">·</span>
              <span className="text-link-ink">
                {data.last_activity_label}
              </span>
              <span className="text-ink-5">·</span>
              <span className="text-ink-3">
                {new Date(
                  data.last_activity_date + 'T00:00:00',
                ).toLocaleDateString('fr-FR', {
                  day: 'numeric',
                  month: 'long',
                  year: 'numeric',
                })}
              </span>
            </div>
          </div>
          {data.persisted ? (
            <span className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-for-soft border border-for-line text-for-ink text-xs font-medium">
              <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
              </svg>
              Sauvegardé
            </span>
          ) : (
            <button
              onClick={() => saveMutation.mutate()}
              disabled={saveMutation.isPending}
              className="shrink-0 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white text-xs font-medium transition-colors"
            >
              {saveMutation.isPending ? (
                <svg className="w-3.5 h-3.5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z" />
                </svg>
              ) : (
                <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
                  <path strokeLinecap="round" strokeLinejoin="round" d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2M12 4v12m0 0l-4-4m4 4l4-4" />
                </svg>
              )}
              Sauvegarder
            </button>
          )}
        </div>
      </div>

      <div className="mb-6">
        <OutcomePanel
          outcome={data.outcome}
          lastActivityLabel={data.last_activity_label}
          lastActivityDate={data.last_activity_date}
        />
      </div>

      {data.curation_status && (
        <div className="flex items-center gap-3 mb-6">
          {(() => {
            const info = curationLabels[data.curation_status] ?? curationLabels.new
            return (
              <span className={`inline-flex items-center px-2.5 py-1 rounded-md border text-xs font-medium ${info.classes}`}>
                {info.label}
              </span>
            )
          })()}
          {data.curation_status !== 'published' && (
            <div className="flex gap-2">
              {data.curation_status !== 'selected' && (
                <button
                  onClick={() => curateMutation.mutate('selected')}
                  disabled={curateMutation.isPending}
                  className="px-3 py-1 rounded text-xs font-medium bg-for-soft border border-for-line text-for-ink hover:bg-for-soft-strong disabled:opacity-50"
                >
                  Sélectionner
                </button>
              )}
              {data.curation_status !== 'dismissed' && (
                <button
                  onClick={() => curateMutation.mutate('dismissed')}
                  disabled={curateMutation.isPending}
                  className="px-3 py-1 rounded text-xs font-medium bg-sunken border border-line-strong text-ink-3 hover:bg-sunken-strong disabled:opacity-50"
                >
                  Écarter
                </button>
              )}
              <button
                onClick={() => curateMutation.mutate('published')}
                disabled={curateMutation.isPending}
                className="px-3 py-1 rounded text-xs font-medium bg-link-soft border border-link-line text-link-ink hover:bg-link-soft-strong disabled:opacity-50"
              >
                Publié
              </button>
            </div>
          )}
        </div>
      )}

      {(data.initiators.length > 0 || data.committee) && (
        <div className="flex flex-wrap gap-3 mb-6">
          {data.initiators.length > 0 && (
            <div className="flex items-center gap-2 text-sm">
              <span className="text-ink-4">Initiateur{data.initiators.length > 1 ? 's' : ''} :</span>
              <div className="flex flex-wrap gap-1.5">
                {data.initiators.map((init, i) => (
                  <span
                    key={i}
                    className="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-sunken text-ink-1 text-xs"
                  >
                    {init.official_url ? (
                      <a
                        href={init.official_url}
                        target="_blank"
                        rel="noreferrer"
                        className="hover:underline"
                      >
                        {init.full_name}
                      </a>
                    ) : (
                      init.full_name
                    )}
                    {init.role && <span className="text-ink-4">{init.role}</span>}
                    {/* Le groupe n'est jamais affiché sans la date à laquelle il a été lu. */}
                    {init.group && init.reference_date && (
                      <span className="text-abstain-ink font-medium" title={init.group.label}>
                        {init.group.abbrev}
                        <span className="text-ink-4 font-normal ml-1">
                          (au{' '}
                          {new Date(init.reference_date + 'T00:00:00').toLocaleDateString('fr-FR', {
                            day: 'numeric',
                            month: 'short',
                            year: 'numeric',
                          })}
                          )
                        </span>
                      </span>
                    )}
                  </span>
                ))}
              </div>
            </div>
          )}
          {data.committee && (
            <div className="flex items-center gap-2 text-sm">
              <span className="text-ink-4">Commission :</span>
              <span className="px-2 py-0.5 rounded-md bg-sunken text-ink-1 text-xs">
                {data.committee}
              </span>
            </div>
          )}
        </div>
      )}

      <div className="grid gap-6 md:grid-cols-2 mb-8">
        <section className="bg-raised border border-line rounded-lg p-5">
          <div className="flex items-center justify-between mb-4">
            <h3 className="text-lg font-semibold">Score</h3>
            <span
              className={`text-3xl font-bold ${scoreTotalColor(data.score.total)}`}
            >
              {data.score.total}
              <span className="text-base text-ink-4 font-normal">/100</span>
            </span>
          </div>
          <div className="space-y-3">
            <ScoreBar
              label="Avancement"
              value={data.score.progress}
              weight={3}
              color="bg-blue-500"
            />
            <ScoreBar
              label="Ampleur"
              value={data.score.magnitude}
              weight={2}
              color="bg-purple-500"
            />
            <ScoreBar
              label="Vélocité"
              value={data.score.momentum}
              weight={1}
              color="bg-cyan-500"
            />
          </div>
          <div className="mt-4 pt-3 border-t border-line">
            <p className="text-xs text-ink-4">
              <span className="text-ink-3 font-medium">Avancement</span> — stade législatif atteint (dépôt → promulgation)
            </p>
            <p className="text-xs text-ink-4 mt-1">
              <span className="text-ink-3 font-medium">Ampleur</span> — importance thématique (budget, santé, sécurité…)
            </p>
            <p className="text-xs text-ink-4 mt-1">
              <span className="text-ink-3 font-medium">Vélocité</span> — nombre d'actes législatifs enregistrés
            </p>
          </div>
        </section>

        <section className="bg-raised border border-line rounded-lg p-5">
          <h3 className="text-lg font-semibold mb-4">Timeline</h3>
          {data.acts.length === 0 ? (
            <p className="text-ink-4 text-sm">Aucun acte enregistré</p>
          ) : (
            (() => {
              // La source les rend du plus ancien au plus récent. On montre les
              // derniers actes en premier: c'est l'état courant du dossier.
              const recentFirst = [...data.acts].reverse()
              const hidden = recentFirst.length - VISIBLE_ACTS
              const shown = allActsShown
                ? recentFirst
                : recentFirst.slice(0, VISIBLE_ACTS)

              return (
                <>
                  <div
                    className={`relative pl-4 border-l border-line-strong space-y-4 ${
                      allActsShown ? 'max-h-56 overflow-y-auto pr-2' : ''
                    }`}
                  >
                    {shown.map((acte, i) => (
                      <div key={`${acte.date}-${i}`} className="relative">
                        <div className="absolute -left-[21px] top-1 w-2.5 h-2.5 rounded-full bg-blue-500 border-2 border-raised" />
                        <p className="text-sm text-ink">{acte.label}</p>
                        <p className="text-xs text-ink-4">
                          {new Date(
                            acte.date + 'T00:00:00',
                          ).toLocaleDateString('fr-FR', {
                            day: 'numeric',
                            month: 'long',
                            year: 'numeric',
                          })}
                        </p>
                      </div>
                    ))}
                  </div>

                  {hidden > 0 && (
                    <button
                      type="button"
                      onClick={() => setAllActsShown(!allActsShown)}
                      aria-expanded={allActsShown}
                      className="mt-4 text-sm text-link-ink hover:text-link-ink underline"
                    >
                      {allActsShown
                        ? 'Masquer les actes antérieurs'
                        : `Afficher les ${hidden} actes antérieurs`}
                    </button>
                  )}
                </>
              )
            })()
          )}
        </section>
      </div>

      <DossierScrutins uid={data.uid} />

      <div className="text-center">
        <a
          href={`https://www.assemblee-nationale.fr/dyn/17/dossiers/${data.uid}`}
          target="_blank"
          rel="noopener noreferrer"
          className="text-sm text-ink-4 hover:text-ink-2 underline"
        >
          Voir sur assemblee-nationale.fr ↗
        </a>
      </div>
    </div>
  )
}
