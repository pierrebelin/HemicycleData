# SPEC — Déploiement VPS Debian 13 + Nginx

Cible : `https://hemicycle.pierrebelin.fr`, VPS Debian 13 (trixie) déjà en service
(Nginx installé, une application Node déjà hébergée). Déploiement automatique à
chaque merge sur `main` via GitHub Actions.

## 1. Décisions

| Sujet | Décision |
|---|---|
| Utilisateur système | `hemicycle`, sans mot de passe, connexion par clé SSH uniquement |
| Emplacement | tout sous `/home/hemicycle` |
| Compilation | sur le VPS (Rust + Node installés sur la machine) |
| Stratégie release | écrasement en place, pas de dossiers horodatés, pas de rollback automatique |
| Migrations | jouées par le binaire au démarrage, `sqlx` ne rejoue jamais une migration déjà appliquée |
| Base | Neon (externe), inchangée |
| Secrets | posés à la main une fois dans `/home/hemicycle/shared/.env`, jamais dans GitHub |
| Port backend | `8085` en local (le `3000` par défaut est supposé pris par l'application Node) |
| Écriture publique | tous les `POST /api/` renvoient `403` sur le vhost public |

## 2. Arborescence cible

```
/home/hemicycle/
  .ssh/
    authorized_keys          → clé publique de déploiement (GitHub Actions)
    id_ed25519               → clé de lecture du dépôt privé (deploy key GitHub)
  app/                       → clone du dépôt, branche main
    target/release/hemicycle-data
    frontend/dist/
  www/                       → copie du build front, servie par Nginx
  shared/
    .env                     → secrets, mode 600
  bin/
    deploy.sh                → script de déploiement (copié depuis app/deploy/)
```

`www/` est distinct de `app/frontend/dist/` pour que Nginx n'ait à lire que ce
dossier : le reste du home reste en `750`, le code source n'est jamais exposé.

## 3. Architecture d'exécution

```
Internet ──► Nginx :443 (vhost public, TLS Let's Encrypt)
               ├─ /            → fichiers statiques /home/hemicycle/www (fallback SPA)
               ├─ POST /api/   → 403
               └─ GET  /api/   → proxy 127.0.0.1:8085

Tunnel SSH ──► Nginx :8080 sur 127.0.0.1 (vhost admin)
               ├─ /            → mêmes fichiers statiques
               └─ /api/        → proxy 127.0.0.1:8085, toutes méthodes

systemd (hemicycle.service, User=hemicycle)
   └─ /home/hemicycle/app/target/release/hemicycle-data
         EnvironmentFile=/home/hemicycle/shared/.env
         PORT=8085
```

### 3.1 Pourquoi ce découpage pour l'écriture

Le front est du JavaScript public : aucune valeur qu'il embarque (jeton, en-tête,
clé) n'est un secret, et `Origin`/`Referer` sont falsifiables par un simple
`curl`. Restreindre les appels « au front seul » n'est pas réalisable.

En revanche, les huit routes en écriture ne sont appelées que par des écrans
d'administration :

| Route | Appelée depuis |
|---|---|
| `POST /api/refresh` | `DossierSelectionPage` |
| `POST /api/registry/refresh` | aucun front (outil) |
| `POST /api/scrutins/refresh` | aucun front (outil) |
| `POST /api/dossiers/{uid}/curate` | `DossierDetailPage`, `DossierSelectionPage` |
| `POST /api/dossiers/{uid}/save` | `DossierDetailPage` |
| `POST /api/themes/extract` | aucun front (outil) |
| `POST /api/themes/propose` | aucun front (outil) |
| `POST /api/themes/arbitrate` | `ThemeArbitrationPage` (déjà protégée par `x-admin-token`) |

Aucun parcours de consultation n'écrit. Le vhost public renvoie donc `403` sur
toute méthode autre que `GET`/`HEAD`/`OPTIONS` sous `/api/`. Sans cela,
n'importe qui déclenche les ingestions Assemblée nationale et consomme la clé
Anthropic.

L'administration passe par un tunnel SSH :

```bash
ssh -N -L 8080:127.0.0.1:8080 hemicycle@<IP_DU_VPS>
```

puis `http://localhost:8080` dans le navigateur. Le vhost admin n'écoute que sur
la boucle locale, il n'est joignable d'aucune façon depuis Internet.

**Dette identifiée, hors périmètre de cette spec** : la protection est
périmétrique (Nginx), pas applicative. Si un jour un autre service du VPS peut
émettre des requêtes vers `127.0.0.1:8085`, il contourne le filtre. Le correctif
propre est un middleware Axum exigeant `ADMIN_TOKEN` sur toutes les routes
d'écriture — `POST /api/themes/arbitrate` montre déjà le motif.

**Deuxième dette** : le binaire écoute sur `0.0.0.0:8085` (`src/main.rs`). Le
pare-feu est donc la seule chose qui empêche d'atteindre l'API en direct sur le
port 8085. Vérifier que `ufw` bloque tout sauf 22/80/443. Correctif propre :
binder `127.0.0.1`.

**Troisième dette** : `CorsLayer::permissive()`. Sans effet ici (front et API
partagent l'origine), conservé tel quel, à resserrer le jour où une origine
tierce existe.

## 4. Préparation du serveur (une seule fois, en root)

### 4.1 Utilisateur

```bash
adduser --disabled-password --gecos "" hemicycle
install -d -m 700 -o hemicycle -g hemicycle /home/hemicycle/.ssh
install -d -m 750 -o hemicycle -g hemicycle /home/hemicycle/www
install -d -m 700 -o hemicycle -g hemicycle /home/hemicycle/shared
install -d -m 750 -o hemicycle -g hemicycle /home/hemicycle/bin
chmod 750 /home/hemicycle
usermod -a -G hemicycle www-data
```

`www-data` (Nginx) entre dans le groupe `hemicycle` pour lire `www/` ; le home en
`750` lui interdit tout le reste.

### 4.2 Droit de redémarrage

Fichier `/etc/sudoers.d/hemicycle`, créé avec `visudo -f`, mode `440` :

```
hemicycle ALL=(root) NOPASSWD: /usr/bin/systemctl restart hemicycle, /usr/bin/systemctl status hemicycle, /usr/bin/systemctl is-active hemicycle
```

Aucun autre droit `sudo`. Vérifier le chemin réel de `systemctl` avec
`command -v systemctl` — un chemin faux dans `sudoers` est une passoire.

### 4.3 Clés SSH

Deux clés distinctes, deux rôles.

**a. Clé d'entrée (GitHub Actions → VPS)** — générée sur le poste local :

```bash
ssh-keygen -t ed25519 -f ~/.ssh/hemicycle_deploy -C "github-actions@hemicycle" -N ""
```

La partie publique va dans `/home/hemicycle/.ssh/authorized_keys` (mode `600`,
propriétaire `hemicycle`). La partie privée va dans le secret GitHub
`VPS_SSH_KEY`. Restreindre la clé dans `authorized_keys` :

```
no-agent-forwarding,no-X11-forwarding,no-user-rc ssh-ed25519 AAAA... github-actions@hemicycle
```

Ne pas mettre `command=` : le workflow a besoin de lancer `deploy.sh` avec un
argument (le SHA).

**b. Clé de lecture du dépôt (VPS → GitHub)** — le dépôt est privé, générée sur
le VPS en tant que `hemicycle` :

```bash
ssh-keygen -t ed25519 -f /home/hemicycle/.ssh/id_ed25519 -N ""
cat /home/hemicycle/.ssh/id_ed25519.pub
```

Ajouter cette clé publique dans GitHub → Settings du dépôt → Deploy keys, en
**lecture seule**. Puis, toujours en tant que `hemicycle` :

```bash
ssh-keyscan github.com >> /home/hemicycle/.ssh/known_hosts
git clone git@github.com:pierrebelin/HemicycleData.git /home/hemicycle/app
```

### 4.4 Chaînes de compilation

En tant que `hemicycle` :

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

Node : la version LTS système suffit si elle est ≥ 20 (Vite 8 l'exige) ;
`node -v` pour vérifier. Si l'application Node déjà présente impose une autre
version, installer `nvm` sous `hemicycle` et poser un `.nvmrc` dans `frontend/`.

### 4.5 Secrets

Fichier `/home/hemicycle/shared/.env`, propriétaire `hemicycle`, mode `600`,
écrit à la main, jamais versionné, jamais dans les secrets GitHub :

```
DATABASE_URL=postgresql://user:password@host.neon.tech/neondb?sslmode=require
ANTHROPIC_API_KEY=sk-ant-...
ADMIN_TOKEN=<chaîne aléatoire longue, ex. openssl rand -hex 32>
PORT=8085
RUST_LOG=info
```

`ANTHROPIC_API_KEY` et `ADMIN_TOKEN` sont facultatifs : sans clé, la
thématisation ne propose rien et le site tourne (RM-01) ; sans jeton, l'écran
d'arbitrage est fermé. Les poser tous les deux.

### 4.6 Port libre

Vérifier avant d'aller plus loin :

```bash
ss -ltnp | grep -E ':(8085|8080)\b'
```

Rien en sortie = les deux ports sont libres. Sinon, changer `PORT` dans `.env`
et les `proxy_pass` des deux vhosts de façon cohérente.

### 4.7 Installation des unités

```bash
install -m 644 /home/hemicycle/app/deploy/systemd/hemicycle.service /etc/systemd/system/
install -m 644 /home/hemicycle/app/deploy/nginx/hemicycle.pierrebelin.fr.conf /etc/nginx/sites-available/
install -m 644 /home/hemicycle/app/deploy/nginx/hemicycle-admin.conf /etc/nginx/sites-available/
ln -sf /etc/nginx/sites-available/hemicycle.pierrebelin.fr.conf /etc/nginx/sites-enabled/
ln -sf /etc/nginx/sites-available/hemicycle-admin.conf /etc/nginx/sites-enabled/
systemctl daemon-reload
systemctl enable hemicycle
```

Premier build manuel avant le premier démarrage (sinon le binaire n'existe pas) :

```bash
sudo -u hemicycle /home/hemicycle/app/deploy/deploy.sh --skip-restart
systemctl start hemicycle
systemctl status hemicycle
```

### 4.8 TLS

Le vhost public livré écoute en HTTP sur le port 80. Poser d'abord
l'enregistrement DNS `A` de `hemicycle.pierrebelin.fr` vers l'IP du VPS, attendre
sa propagation (`dig +short hemicycle.pierrebelin.fr`).

Méthode **webroot**, pas le plugin `--nginx` en authentificateur : tant que
`www/` est vide, le fallback SPA `try_files … /index.html` renvoie `404` et le
bloc temporaire injecté par le plugin ne prend pas la main — l'émission échoue
avec `Invalid response from …/.well-known/acme-challenge/… : 404`. Le vhost
livré contient un `location ^~ /.well-known/acme-challenge/` permanent qui règle
le problème et sert aussi aux renouvellements.

```bash
mkdir -p /var/www/certbot/.well-known/acme-challenge
```

Vérifier que le chemin est servi avant d'appeler l'autorité :

```bash
echo ok > /var/www/certbot/.well-known/acme-challenge/probe
curl -i http://hemicycle.pierrebelin.fr/.well-known/acme-challenge/probe   # 200 attendu
```

Puis émettre et installer le certificat :

```bash
certbot run -a webroot -w /var/www/certbot -i nginx -d hemicycle.pierrebelin.fr --redirect --agree-tos -m contact@pierrebelin.fr
rm /var/www/certbot/.well-known/acme-challenge/probe
```

Certbot réécrit le vhost en place : il ajoute le bloc `listen 443 ssl`, les
chemins de certificat et la redirection 80 → 443. Le renouvellement est assuré
par le timer `certbot.timer` (`systemctl list-timers certbot.timer` pour
vérifier).

**Conséquence sur les mises à jour** : ne jamais réinstaller
`hemicycle.pierrebelin.fr.conf` par-dessus la version modifiée par Certbot sans
rejouer `certbot --nginx` derrière. Le déploiement automatique ne touche pas aux
fichiers Nginx — seule une intervention manuelle peut casser cela.

## 5. Déploiement automatique

### 5.1 Déclencheur

`push` sur `main` (donc tout merge de pull request), plus `workflow_dispatch`
pour un déclenchement manuel.

### 5.2 Étapes

1. **Job `test`** (runner GitHub `ubuntu-latest`) — `cargo test` (197 tests),
   `oxlint`, puis `npm run build` qui enchaîne `tsc -b` et `vite build`. Aucun
   accès à la base : le code n'utilise aucune macro `sqlx::query!` vérifiée à la
   compilation, tout passe par les variantes runtime, et les tests de use case
   reposent sur des fakes in-memory.

   `cargo fmt --check` et `cargo clippy -- -D warnings` sont **volontairement
   absents du verrou** : à ce jour le code ne les passe pas (13 erreurs clippy,
   formatage non conforme). Les mettre dans le verrou bloquerait tout
   déploiement dès le premier merge. À rétablir une fois la dette résorbée.
2. **Job `deploy`** — s'exécute uniquement si `test` est vert et si la référence
   est `refs/heads/main`. Se connecte en SSH sous `hemicycle` et lance
   `~/app/deploy/deploy.sh <sha>`.

Le job `deploy` déclare `concurrency: hemicycle-production` avec
`cancel-in-progress: false` : deux merges rapprochés se déploient l'un après
l'autre, jamais en parallèle sur le même dossier.

### 5.3 Ce que fait `deploy.sh` sur le VPS

1. `git fetch --prune`, puis `git reset --hard <sha>` sur `app/` — l'état du
   serveur est exactement celui du commit, toute modification locale est écrasée.
2. `cargo build --release --locked`.
3. `npm ci` puis `npm run build` dans `frontend/`.
4. `rsync -a --delete frontend/dist/ ~/www/`.
5. `sudo systemctl restart hemicycle`.
6. Attente active de `GET 127.0.0.1:8085/api/health` (30 tentatives, 1 s). Échec
   du health check = sortie non nulle = job GitHub rouge.

### 5.4 Indisponibilité et retour arrière

Écrasement en place, assumé :

- Le redémarrage du service coupe l'API environ une seconde.
- `rsync --delete` sur `www/` ouvre une fenêtre de quelques centaines de
  millisecondes où un visiteur peut recevoir un `index.html` neuf pointant vers
  des fichiers d'actifs déjà supprimés. Il obtient une page cassée jusqu'au
  rechargement. Acceptable au trafic visé ; c'est le prix de l'écrasement.
- Pas de rollback automatique. Retour arrière manuel :

```bash
ssh hemicycle@<IP_DU_VPS> '~/app/deploy/deploy.sh <sha_precedent>'
```

Une migration déjà appliquée n'est pas défaite par ce retour arrière — voir §5.5.

### 5.5 Migrations

`sqlx::migrate!()` s'exécute au démarrage du binaire
(`src/infrastructure/config.rs`). La table `_sqlx_migrations` tient le journal :
une migration déjà appliquée n'est jamais rejouée, seules les nouvelles passent.

Aucune sauvegarde n'est prise avant. En clair : une migration destructrice part
en production sans filet, et le retour arrière du code ne la défait pas. Neon
fournit du *point-in-time restore* — c'est le seul recours en cas de dégât.
Contrainte de travail à respecter : écrire des migrations additives (ajout de
colonne nullable, nouvelle table), jamais de `DROP` ni de `ALTER` réducteur sans
étape de transition.

### 5.6 Secrets GitHub

| Secret | Contenu |
|---|---|
| `VPS_HOST` | IP ou nom d'hôte du VPS |
| `VPS_SSH_KEY` | clé privée de §4.3.a, contenu intégral y compris les lignes `-----BEGIN/END-----` |
| `VPS_KNOWN_HOSTS` | sortie de `ssh-keyscan <IP_DU_VPS>` |

`VPS_KNOWN_HOSTS` n'est pas cosmétique : sans lui, le workflow accepterait
n'importe quelle clé d'hôte et livrerait la clé de déploiement au premier
intercepteur venu. Aucun secret applicatif ne transite par GitHub.

### 5.7 Fichiers livrés

| Fichier | Rôle |
|---|---|
| `.github/workflows/deploy.yml` | verrou de tests + déploiement SSH |
| `deploy/deploy.sh` | script exécuté sur le VPS (build, publication, restart, health check) |
| `deploy/systemd/hemicycle.service` | unité systemd, à installer dans `/etc/systemd/system/` |
| `deploy/nginx/hemicycle.pierrebelin.fr.conf` | vhost public, à installer dans `/etc/nginx/sites-available/` |
| `deploy/nginx/hemicycle-admin.conf` | vhost d'administration sur `127.0.0.1:8080` |

Ordre de mise en service : §4 en entier (dont `certbot`, §4.8) **avant** le
premier merge sur `main`. L'étape « Vérification publique » du workflow
interroge `https://hemicycle.pierrebelin.fr` et échoue tant que le certificat
n'est pas posé.

## 6. Exploitation

```bash
sudo systemctl status hemicycle          # état
journalctl -u hemicycle -f               # logs en direct
journalctl -u hemicycle --since "1 hour ago" -p err
curl -s 127.0.0.1:8085/api/health        # santé côté serveur
```

Vérification du filtre d'écriture depuis l'extérieur — doit répondre `403` :

```bash
curl -s -o /dev/null -w '%{http_code}\n' -X POST https://hemicycle.pierrebelin.fr/api/refresh
```

## 7. Recette

- [ ] `https://hemicycle.pierrebelin.fr` répond en 200, certificat valide
- [ ] Une route SPA profonde rechargée directement (F5) renvoie l'application, pas un 404
- [ ] `GET /api/health` répond via HTTPS
- [ ] `POST /api/refresh` depuis Internet renvoie `403`
- [ ] Le port `8085` n'est pas joignable depuis l'extérieur (`nc -zv <IP> 8085` échoue)
- [ ] Tunnel SSH ouvert, `http://localhost:8080` sert le site et accepte les POST
- [ ] `systemctl restart hemicycle` remonte le service et rejoue les migrations sans erreur
- [ ] Un merge sur `main` déclenche le workflow et le site sert bien le nouveau code
- [ ] `sudo -l` sous `hemicycle` ne liste que les trois commandes `systemctl`
