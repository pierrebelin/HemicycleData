# Références vérifiées vote → texte

`official-text-versions.json` contient uniquement les versions de texte dont
une publication de l'Assemblée nationale établit explicitement le lien avec un
scrutin public sur l'ensemble.

`document_uid` est l'identifiant public visible dans l'URL du document (par
exemple `l17t0029`) ; il n'est pas déduit d'un titre et il permet de retrouver
la même version dans la publication de l'Assemblée.

Une entrée doit porter le numéro et la législature du scrutin, jamais son UID
interne : l'import exécuté sur le VPS résout cet identifiant public au moment
où la base contient le scrutin. Il refuse un numéro absent ou ambigu plutôt
que de créer un rattachement approximatif.

Ne pas créer d'entrée à partir d'une similarité de titre, de date ou de dossier.
`mapping_source_url` doit désigner l'acte officiel qui établit le lien ;
`official_url` désigne la version précise à lire ; `content_url` désigne sa
version Open Data HTML, explicitement liée depuis cette page. Une correction
consiste à modifier l'entrée puis à relancer les outils ; ils sont idempotents.

Lancer sur le VPS, après le rafraîchissement des scrutins :

```bash
cargo run --bin sync-official-text-versions
cargo run --bin capture-official-text-versions
```

La commande lit `DATABASE_URL` uniquement dans l'environnement du VPS. Elle
ne doit pas être exécutée depuis un poste qui n'a pas accès à cette base. La
capture vient après la synchronisation : elle conserve le HTML officiel brut,
une version texte dérivée et l'empreinte du document. Rien n'est exposé au
lecteur tant que l'étape de réutilisation et d'affichage n'est pas réalisée.
