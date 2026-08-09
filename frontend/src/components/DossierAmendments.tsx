import { useState } from 'react'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { Card, ErrorPanel, Note, Pill, SectionTitle, type PillTone } from './ui'
import type { AmendmentDto, DossierAmendmentsResponse } from '../types/amendments'

const PAGE_SIZE = 25

/**
 * Teinte du sort. Même échelle que les dossiers : adopté et rejeté se
 * distinguent, tout le reste reste neutre. Un sort hors référentiel (`other`)
 * est neutre lui aussi — il n'a pas à ressembler à un rejet parce qu'on ne
 * sait pas le classer.
 */
const fateTones: Record<string, PillTone> = {
  adopted: 'yes',
  rejected: 'no',
}

function formatDate(iso: string) {
  return new Date(iso + 'T00:00:00').toLocaleDateString('fr-FR', {
    day: 'numeric',
    month: 'short',
    year: 'numeric',
  })
}

/**
 * Retire le balisage de l'exposé sommaire, sans toucher aux mots.
 *
 * La source publie l'exposé avec un peu de HTML. Le rendre tel quel afficherait
 * des balises au milieu du texte ; l'injecter dans le DOM ferait confiance à
 * une chaîne venue du réseau. On enlève donc les balises et rien d'autre :
 * aucun mot ajouté, retiré ni réordonné (README.md §6, RM-03).
 *
 * C'est un repli. La règle appartient au serveur, sous forme de liste blanche
 * de balises conservées — à écrire quand H9 aura dit quel balisage la source
 * emploie réellement (SPEC-amendements §6).
 */
function withoutMarkup(raw: string) {
  return raw
    .replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/p>/gi, '\n\n')
    .replace(/<[^>]+>/g, '')
    .replace(/&nbsp;/gi, ' ')
    .replace(/&amp;/gi, '&')
    .replace(/&lt;/gi, '<')
    .replace(/&gt;/gi, '>')
    .replace(/&#39;|&apos;/gi, "'")
    .replace(/&quot;/gi, '"')
    .replace(/\n{3,}/g, '\n\n')
    .trim()
}

/** Un signataire nommé, avec le groupe qu'il avait au dépôt. */
function Author({ amendment }: { amendment: AmendmentDto }) {
  const name = amendment.author_name ?? amendment.author_actor_uid ?? 'Auteur non identifié'

  return (
    <span className="text-ink-soft">
      {amendment.author_official_url ? (
        <a
          href={amendment.author_official_url}
          target="_blank"
          rel="noreferrer"
          className="font-medium text-ink hover:underline"
        >
          {name}
        </a>
      ) : (
        <span className="font-medium text-ink">{name}</span>
      )}

      {amendment.author_group_abbrev && (
        <span
          className="ml-1.5"
          title={
            amendment.author_group_origin === 'resolved_at_deposit'
              ? `${amendment.author_group_label} — groupe du signataire à la date de dépôt, reconstitué depuis les appartenances datées`
              : `${amendment.author_group_label} — groupe publié par la source sur cet amendement`
          }
        >
          ({amendment.author_group_abbrev})
        </span>
      )}

      {amendment.author_group_ambiguous && (
        <span
          className="ml-1.5 text-ink-faint"
          title="Deux groupes concurrents revendiquent ce signataire à la date de dépôt : aucun n'est affiché."
        >
          (groupe indéterminé)
        </span>
      )}

      {amendment.cosignatory_count > 0 && (
        <span className="text-ink-faint">
          {' '}
          et {amendment.cosignatory_count} cosignataire
          {amendment.cosignatory_count > 1 ? 's' : ''}
        </span>
      )}
    </span>
  )
}

function AmendmentRow({ amendment }: { amendment: AmendmentDto }) {
  const summary = amendment.summary ? withoutMarkup(amendment.summary) : null

  return (
    <Card className="px-4 py-3">
      <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
        <span className="font-mono text-xs text-ink-faint">n° {amendment.number}</span>
        <span className="text-sm font-medium text-ink">{amendment.target_title}</span>
        {amendment.fate_label && (
          <Pill tone={fateTones[amendment.fate_code] ?? 'neutral'}>
            {amendment.fate_label}
          </Pill>
        )}
        {amendment.deposited_on && (
          <span className="text-xs text-ink-faint">
            déposé le {formatDate(amendment.deposited_on)}
          </span>
        )}
      </div>

      <p className="mt-1 text-sm">
        <Author amendment={amendment} />
      </p>

      {summary ? (
        <details className="mt-2">
          <summary className="cursor-pointer text-xs font-medium text-ink-soft hover:text-ink">
            Exposé sommaire
          </summary>
          {/* Verbatim : le texte du signataire, ni résumé ni abrégé (RM-03). */}
          <p className="mt-1.5 whitespace-pre-wrap text-sm leading-relaxed text-ink-soft">
            {summary}
          </p>
        </details>
      ) : (
        <p className="mt-2 text-xs text-ink-faint">
          La source ne publie pas d'exposé sommaire pour cet amendement.
        </p>
      )}
    </Card>
  )
}

/**
 * Section amendements d'un dossier. Toujours présente, même vide : une section
 * absente laisserait croire qu'aucun amendement n'a été déposé, alors que la
 * source peut simplement ne rattacher aucun texte de ce dossier.
 *
 * L'ordre est celui du dépôt, la borne est de {PAGE_SIZE} par page, et le total
 * est affiché : paginer n'est pas filtrer (README.md §2, RM-07). Aucun tri par
 * « importance » ni par nombre de cosignataires — ce serait un classement.
 */
export default function DossierAmendments({ uid }: { uid: string }) {
  const [offset, setOffset] = useState(0)

  const { data, isLoading, isError, error } = useQuery<DossierAmendmentsResponse>({
    queryKey: ['dossier-amendements', uid, offset],
    queryFn: () =>
      fetch(`/api/dossiers/${uid}/amendements?limit=${PAGE_SIZE}&offset=${offset}`).then(
        (res) => {
          if (!res.ok) throw new Error(`HTTP ${res.status}`)
          return res.json()
        },
      ),
    enabled: !!uid,
    placeholderData: keepPreviousData,
  })

  const hasPrevious = offset > 0
  const hasNext = data ? offset + data.count < data.total : false

  return (
    <section className="mb-6">
      <SectionTitle count={data && data.total > 0 ? data.total : undefined}>
        Amendements
      </SectionTitle>

      {isLoading && (
        <p className="animate-pulse text-sm text-ink-faint">
          Chargement des amendements…
        </p>
      )}

      {isError && <ErrorPanel error={error} />}

      {data && (
        <div className="space-y-2">
          {data.total === 0 ? (
            <p className="rounded-lg border border-line bg-surface px-4 py-3 text-sm text-ink-soft">
              La source ne rattache aucun amendement à un texte de ce dossier.
              Cela ne signifie pas qu'aucun amendement n'a été déposé sur ce
              texte.
            </p>
          ) : (
            <>
              {data.amendments.map((amendment) => (
                <AmendmentRow key={amendment.uid} amendment={amendment} />
              ))}

              <div className="flex items-center justify-between gap-3 pt-1">
                <span className="text-xs text-ink-faint">
                  {offset + 1}–{offset + data.count} sur {data.total}
                </span>
                <div className="flex gap-2">
                  <button
                    type="button"
                    disabled={!hasPrevious}
                    onClick={() => setOffset(Math.max(0, offset - PAGE_SIZE))}
                    className="rounded-md border border-line px-2.5 py-1 text-xs text-ink-soft disabled:opacity-40 enabled:hover:bg-surface-soft"
                  >
                    Précédents
                  </button>
                  <button
                    type="button"
                    disabled={!hasNext}
                    onClick={() => setOffset(offset + PAGE_SIZE)}
                    className="rounded-md border border-line px-2.5 py-1 text-xs text-ink-soft disabled:opacity-40 enabled:hover:bg-surface-soft"
                  >
                    Suivants
                  </button>
                </div>
              </div>
            </>
          )}

          <Note>
            {data.source_note} {data.pagination_note} {data.coverage_note}
            {data.coverage.without_summary > 0 && (
              <>
                {' '}
                {data.coverage.without_summary} amendement
                {data.coverage.without_summary > 1 ? 's' : ''} de ce dossier
                {data.coverage.without_summary > 1 ? ' sont déposés' : ' est déposé'}{' '}
                sans exposé sommaire publié.
              </>
            )}
          </Note>
        </div>
      )}
    </section>
  )
}
