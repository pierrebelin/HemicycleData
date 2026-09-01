-- Un script de peuplement peut etre rejoue sans dupliquer les memes extraits
-- attribues a une meme source. Les ajouts manuels venant d'une autre source
-- restent independants.
CREATE UNIQUE INDEX uq_candidate_program_proposals_source_excerpt
    ON candidate_program_proposals(candidate_id, family_code, excerpt, source_url);
