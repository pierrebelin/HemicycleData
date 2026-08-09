/** Origine du groupe affiché à côté d'un signataire (README.md §3.2). */
export type GroupOrigin = 'published' | 'resolved_at_deposit' | 'unknown'

export interface AmendmentDto {
  uid: string
  number: string
  target_title: string
  target_kind: string | null
  author_kind: 'deputy' | 'institutional'
  author_actor_uid: string | null
  author_name: string | null
  author_official_url: string | null
  /** Groupe du signataire **à la date de dépôt**, jamais son groupe actuel. */
  author_group_label: string | null
  author_group_abbrev: string | null
  author_group_origin: GroupOrigin
  author_group_ambiguous: boolean
  fate_code: string
  fate_label: string
  state_label: string | null
  deposited_on: string | null
  /** Exposé sommaire entier. Le serveur ne le tronque jamais (RM-03). */
  summary: string | null
  cosignatory_count: number
}

export interface AmendmentCoverageDto {
  total: number
  without_summary: number
  unknown_fates: number
}

export interface DossierAmendmentsResponse {
  total: number
  count: number
  offset: number
  limit: number
  amendments: AmendmentDto[]
  coverage: AmendmentCoverageDto
  coverage_note: string
  pagination_note: string
  source_note: string
}
