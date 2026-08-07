import type {
  GroupBreakdownDto,
  TallyDto,
  VoteDto,
} from '../types/scrutins'

/** Couleur de remplissage d'un siège, par position de vote. */
export const REMPLISSAGE_PAR_POSITION: Record<VoteDto['position'], string> = {
  for: 'fill-for-ink',
  against: 'fill-against-ink',
  abstention: 'fill-abstain-ink',
  not_voting: 'fill-ink-5',
}

export const TEXTE_PAR_POSITION: Record<VoteDto['position'], string> = {
  for: 'text-for-ink',
  against: 'text-against-ink',
  abstention: 'text-abstain-ink',
  not_voting: 'text-ink-4',
}

/**
 * Vote nominal augmenté de son groupe. Le groupe est porté par le bloc, pas par
 * le vote : sans cette reprise, une infobulle de siège ne pourrait pas nommer le
 * groupe du député — et §3.1 interdit de le déduire d'autre chose.
 */
export type SiegeVote = VoteDto & {
  groupAbbrev: string | null
  groupLabel: string | null
}

export function siegesDesGroupes(groups: GroupBreakdownDto[]): SiegeVote[] {
  return groups.flatMap((groupe) =>
    groupe.votes.map((vote) => ({
      ...vote,
      groupAbbrev: groupe.abbrev,
      groupLabel: groupe.label,
    })),
  )
}

/**
 * Décompte reconstitué depuis les positions nominales. Sert à ce que la légende
 * dise exactement ce que le dessin montre : la synthèse officielle compte les
 * non-votants volontaires à part, la liste nominale les range avec les autres
 * non-votants, et deux totaux différents sous un même graphique seraient une
 * incohérence affichée.
 */
export function tallyDesVotes(votes: Pick<VoteDto, 'position'>[]): TallyDto {
  return {
    votes_for: votes.filter((v) => v.position === 'for').length,
    votes_against: votes.filter((v) => v.position === 'against').length,
    abstentions: votes.filter((v) => v.position === 'abstention').length,
    not_voting: votes.filter((v) => v.position === 'not_voting').length,
    voluntary_not_voting: 0,
  }
}

export type Position = {
  cle: string
  libelle: string
  /** Accordé sur l'effectif : « 1 abstention », « 2 abstentions ». */
  libelleAccorde: string
  total: number
  /** Classes écrites en toutes lettres : Tailwind ne voit pas une classe construite. */
  fond: string
  texte: string
}

/**
 * Les quatre positions publiées par l'Assemblée, dans l'ordre d'affichage
 * retenu partout sur le site. Les non-votants volontaires sont additionnés aux
 * non-votants : la source les distingue, la répartition graphique ne le peut
 * pas sans introduire une cinquième couleur pour un effectif d'ordinaire nul.
 */
export function positions(tally: TallyDto): Position[] {
  const nonVotants = tally.not_voting + tally.voluntary_not_voting
  return [
    {
      cle: 'for',
      libelle: 'Pour',
      libelleAccorde: 'pour',
      total: tally.votes_for,
      fond: 'bg-for-ink',
      texte: 'text-for-ink',
    },
    {
      cle: 'abstention',
      libelle: 'Abstention',
      libelleAccorde: tally.abstentions > 1 ? 'abstentions' : 'abstention',
      total: tally.abstentions,
      fond: 'bg-abstain-ink',
      texte: 'text-abstain-ink',
    },
    {
      cle: 'not_voting',
      libelle: 'Non-votants',
      libelleAccorde: nonVotants > 1 ? 'non-votants' : 'non-votant',
      total: nonVotants,
      fond: 'bg-ink-5',
      texte: 'text-ink-4',
    },
    {
      cle: 'against',
      libelle: 'Contre',
      libelleAccorde: 'contre',
      total: tally.votes_against,
      fond: 'bg-against-ink',
      texte: 'text-against-ink',
    },
  ]
}
