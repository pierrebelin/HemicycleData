export interface FamilyDto {
  code: string
  label: string
  scope: string
}

export interface FamiliesResponse {
  families: FamilyDto[]
  max_families_per_text: number
  method_note: string
}

export interface AssignedFamilyDto {
  code: string
  label: string
  /** `proposal` ou `human_arbitration`. */
  origin: string
  /** Mention affichée à côté du rattachement (RM-09). */
  origin_note: string
  opened_on: string
  motive: string | null
}

export interface TextSummaryDto {
  key: string
  label: string
  scrutin_count: number
  first_vote: string | null
  last_vote: string | null
  dossier_uid: string | null
  dossier_label: string | null
  families: AssignedFamilyDto[]
  /** `no_family` : le modèle n'a rien retenu. `failed` : il n'a pas répondu. */
  last_attempt_outcome: string | null
}

export interface TextListResponse {
  items: TextSummaryDto[]
  total: number
  offset: number
  method_note: string
}

export interface TextScrutinDto {
  uid: string
  number: string
  date: string
  subject: string
  outcome_label: string
  votes_for: number
  votes_against: number
  abstentions: number
}

export interface AssignmentHistoryDto {
  code: string
  label: string
  origin: string
  origin_note: string
  opened_on: string
  closed_on: string | null
  author: string
  motive: string | null
}

export interface ProposedFamilyDto {
  code: string
  label: string
  justification: string
}

export interface ProposalDto {
  model: string
  prompt_version: string
  produced_on: string
  families: ProposedFamilyDto[]
}

export interface TextDetailResponse extends TextSummaryDto {
  scrutins: TextScrutinDto[]
  history: AssignmentHistoryDto[]
  proposal: ProposalDto | null
  method_note: string
}

export interface FamilyCoverageDto {
  code: string
  label: string
  text_count: number
  scrutin_count: number
  arbitrated_text_count: number
}

export interface MethodResponse {
  families: FamilyCoverageDto[]
  max_families_per_text: number
  texts_total: number
  texts_assigned: number
  texts_arbitrated: number
  texts_awaiting_arbitration: number
  texts_without_family: number
  texts_attempt_failed: number
  texts_never_attempted: number
  scrutins_total: number
  scrutins_with_text: number
  scrutins_assigned: number
  scrutins_without_text: number
  dossiers_total: number
  dossiers_linked_to_text: number
  dossiers_assigned: number
  extraction_rule: string
  model_scope: string
  method_note: string
}
