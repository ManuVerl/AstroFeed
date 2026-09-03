# AstroFeed — Spécification fonctionnelle et technique

> Version : 0.1.0-draft  
> Date : 2025  
> Statut : **Draft**

---

## 1. Présentation

**AstroFeed** est une application de bureau multiplateforme (Linux, Windows) écrite en **Rust**, permettant de suivre et d'afficher les événements **astronomiques** et **radioastronomiques** visibles ou audibles depuis une ou plusieurs positions géographiques définies par l'utilisateur.

---

## 2. Objectifs

| # | Objectif |
|---|----------|
| 1 | Afficher les événements astronomiques et radioastronomiques à venir et passés pour une position donnée |
| 2 | Permettre la gestion de plusieurs positions nommées et géolocalisées |
| 3 | Récupérer automatiquement les données depuis des sources publiques et gratuites |
| 4 | Rester utilisable (UI non bloquée) même si les sources externes sont indisponibles |
| 5 | Proposer un paramétrage minimal (thème, fréquence de mise à jour) |

---

## 3. Fonctionnalités

### 3.1 Gestion des positions

Une **position** est caractérisée par :

| Champ | Type | Description |
|-------|------|-------------|
| `name` | `String` | Nom libre (ex : « Domicile », « Observatoire ») |
| `icon` | `Enum` | Icône parmi une sélection prédéfinie : 🏠 Maison, 🔭 Observatoire, 🏕️ Terrain, 📡 Station, 🏔️ Montagne |
| `latitude` | `f64` | Latitude en degrés décimaux (−90 … +90) |
| `longitude` | `f64` | Longitude en degrés décimaux (−180 … +180) |
| `acquisition` | `Enum` | `GPS` (position automatique du poste) ou `Manual` (saisie clavier) |

- L'utilisateur peut enregistrer **n** positions.
- Une position est sélectionnée comme **position active** pour le calcul des événements.
- La suppression d'une position active redirige vers une autre position ou demande d'en créer une.

### 3.2 Catalogue des événements

Chaque événement possède les attributs communs suivants :

| Champ | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Identifiant unique |
| `title` | `String` | Intitulé de l'événement |
| `category` | `Enum` | `Astronomical` 🔭 ou `RadioAstronomical` 📡 |
| `event_type` | `Enum` | Sous-type (voir §3.2.1 et §3.2.2) |
| `start_time` | `DateTime<Utc>` | Début de l'événement (UTC) |
| `end_time` | `DateTime<Utc>` | Fin de l'événement (UTC) |
| `sky_position` | `SkyCoord` | Azimut + élévation au-dessus de la position |
| `equipment` | `Option<String>` | Matériel suggéré (ex : « télescope 150mm ») |
| `source` | `String` | Identifiant de la source externe ayant fourni l'événement |
| `description` | `Option<String>` | Description courte facultative |

Les événements **radioastronomiques** ajoutent :

| Champ | Type | Description |
|-------|------|-------------|
| `freq_min_mhz` | `f64` | Fréquence basse en MHz |
| `freq_max_mhz` | `f64` | Fréquence haute en MHz |
| `listen_direction` | `Option<SkyCoord>` | Direction d'écoute (azimut/élévation) |

#### 3.2.1 Événements astronomiques (`Astronomical` 🔭)

| Sous-type | Description |
|-----------|-------------|
| `ISS_Flyover` | Survol de la position par l'ISS |
| `CometVisible` | Passage d'une comète visible à l'œil nu ou au télescope |
| `MeteorShower` | Pluie de météores notable |
| `PlanetVisible` | Planète du système solaire visible |
| `Other` | Tout autre événement notable |

#### 3.2.2 Événements radioastronomiques (`RadioAstronomical` 📡)

| Sous-type | Description |
|-----------|-------------|
| `ISS_Radio` | Programme de radiocommunication ARISS avec l'ISS |
| `SolarTransit` | Transit du Soleil au-dessus de la position |
| `CometTransit` | Transit d'une comète (fréquence + axe d'écoute) |
| `Other` | Tout autre événement radio notable |

### 3.3 Fenêtre temporelle d'affichage

| Sens | Durée |
|------|-------|
| Futur | Jusqu'à **+1 an** depuis aujourd'hui |
| Passé | Jusqu'à **−1 mois** depuis aujourd'hui |

Les événements sont triés par **ordre chronologique croissant** (le prochain en haut).

### 3.4 Sources externes de données

Les données sont acquises depuis des API publiques et gratuites. Le tableau ci-dessous liste les sources candidates (à confirmer/compléter à l'implémentation) :

| Source | URL | Données fournies |
|--------|-----|-----------------|
| **Heavens-Above** | `https://www.heavens-above.com` | Survols ISS, planètes |
| **NASA Spot the Station** | `https://spotthestation.nasa.gov` | Survols ISS (flux RSS) |
| **Open-Notify ISS** | `http://api.open-notify.org/iss-pass.json` | Passes ISS |
| **JPL Horizons** | `https://ssd.jpl.nasa.gov/api/horizons.api` | Éphémérides planètes, comètes |
| **ARISS** | `https://www.ariss.org` | Contacts radio ISS (RSS) |
| **IMO Meteor Calendar** | `https://www.imo.net` | Calendrier pluies de météores |
| **Minor Planet Center** | `https://minorplanetcenter.net` | Comètes récentes |

> **Règle impérative** : l'appel aux sources est effectué en tâche de fond (thread/async). L'UI reste responsive en permanence.

#### 3.4.1 Rapport de synchronisation

Un écran/panneau dédié (accessible via le menu ou une icône d'état) liste pour chaque source :

| Champ | Description |
|-------|-------------|
| `source_name` | Nom de la source |
| `status` | `OK` ✅ / `Error` ❌ / `Pending` 🔄 |
| `last_sync` | Date et heure de la dernière synchronisation réussie |
| `error_message` | Message d'erreur le cas échéant |

### 3.5 Affichage des événements

- **Vue liste** : liste chronologique avec icône de catégorie, titre, date/heure locale, position dans le ciel.
- **Vue détail** : panneau latéral ou fenêtre modale affichant tous les champs de l'événement.
- **Filtres** :
  - Par catégorie : Astronomique 🔭 / Radioastronomique 📡 / Tous
  - Par sous-type d'événement
  - Par position (si plusieurs positions enregistrées)
  - Passés / À venir / Tous

### 3.6 Paramétrage

| Paramètre | Valeurs possibles | Défaut |
|-----------|------------------|--------|
| Thème | `Dark` 🌑 / `Light` ☀️ | `Dark` |
| Fréquence de mise à jour | `OnStartup` / `Weekly` / `Monthly` | `OnStartup` |
| Position active | Une des positions enregistrées | Première créée |
| Format d'heure | `UTC` / `Local` | `Local` |

---

## 4. Architecture technique

### 4.1 Langage et plateforme

| Élément | Choix |
|---------|-------|
| Langage | **Rust** (édition 2021+) |
| Cibles | `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` |
| Toolchain | `stable` |

### 4.2 Framework UI — analyse comparative

Trois frameworks Rust sont candidats. Le choix final sera arrêté lors du démarrage de l'implémentation.

| Framework | Avantages | Inconvénients | Score |
|-----------|-----------|---------------|-------|
| **egui / eframe** | Immédiat (immediate-mode), multiplateforme, léger, pas de dépendances système | Look natif limité, rendu OpenGL/wgpu | ⭐⭐⭐ |
| **Tauri** | HTML/CSS/JS pour l'UI, webview natif, très actif | Dépendance webview OS, écosystème JS | ⭐⭐ |
| **Slint** | DSL déclaratif, animations, look moderne, embarqué-friendly | Moins de composants prêts, DSL à apprendre | ⭐⭐⭐ |

> **Recommandation initiale : `egui/eframe`** pour la rapidité de prototypage et l'absence de dépendances système lourdes.  
> À réévaluer si des besoins de rendu complexe (carte du ciel, graphiques polaires) émergent.

### 4.3 Crates principales envisagées

| Crate | Usage |
|-------|-------|
| `eframe` / `egui` | Framework UI |
| `tokio` | Runtime async pour les appels réseau |
| `reqwest` | Client HTTP async |
| `serde` / `serde_json` | Sérialisation/désérialisation |
| `chrono` | Manipulation des dates et heures |
| `uuid` | Génération d'identifiants uniques |
| `dirs` | Chemins de configuration utilisateur cross-platform |
| `toml` | Persistance de la configuration |
| `astro` ou `sgp4` | Calculs orbitaux (passes ISS, éphémérides) |
| `log` + `env_logger` | Journalisation |

### 4.4 Structure du projet

```
astrofeed/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── SPEC.md
├── src/
│   ├── main.rs               # Point d'entrée, init UI
│   ├── app.rs                # État global de l'application (App struct)
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── main_window.rs    # Fenêtre principale, layout
│   │   ├── event_list.rs     # Vue liste des événements
│   │   ├── event_detail.rs   # Vue détail d'un événement
│   │   ├── positions.rs      # Gestion des positions
│   │   ├── settings.rs       # Panneau de paramétrage
│   │   └── sync_report.rs    # Rapport de synchronisation des sources
│   ├── model/
│   │   ├── mod.rs
│   │   ├── event.rs          # Structs Event, SkyCoord, EventType, Category
│   │   └── position.rs       # Struct Position, PositionIcon
│   ├── sources/
│   │   ├── mod.rs
│   │   ├── manager.rs        # Orchestration des sources, rapport d'état
│   │   ├── iss_passes.rs     # Source : passes ISS
│   │   ├── planets.rs        # Source : planètes visibles
│   │   ├── meteors.rs        # Source : pluies de météores
│   │   ├── comets.rs         # Source : comètes
│   │   └── iss_radio.rs      # Source : contacts radio ARISS
│   ├── config/
│   │   ├── mod.rs
│   │   └── settings.rs       # Lecture/écriture TOML de la config
│   └── utils/
│       ├── mod.rs
│       └── geo.rs            # Conversion coordonnées, calculs astronomiques
├── assets/
│   └── icons/                # Icônes SVG/PNG embarquées
└── tests/
    └── integration/
```

### 4.5 Persistance

- **Configuration** : fichier `config.toml` dans le répertoire de configuration utilisateur (`dirs::config_dir()`).
- **Cache événements** : fichier `events_cache.json` dans le répertoire de données utilisateur (`dirs::data_dir()`).
- **Log de synchronisation** : `sync_log.json` dans le même répertoire de données.

### 4.6 Flux de données

```
┌─────────────────────────────────────────────────┐
│                   UI Thread (egui)              │
│  EventListView ◄──── AppState ◄──── EventStore  │
└────────────────────────────┬────────────────────┘
                             │ channel (mpsc)
                             ▼
┌─────────────────────────────────────────────────┐
│              Background Task (tokio)            │
│  SourceManager → [Source1, Source2, …]          │
│       │                                         │
│       └─► HTTP requests → parse → normalize     │
└─────────────────────────────────────────────────┘
```

---

## 5. Expérience utilisateur

### 5.1 Thèmes

| Thème | Palette de base |
|-------|----------------|
| **Dark** | Fond `#1a1a2e`, surfaces `#16213e`, texte `#e0e0e0`, accent `#0f3460` |
| **Light** | Fond `#ffffff`, surfaces `#f0f4f8`, texte `#1a1a2e`, accent `#3b82f6` |

Le thème sombre est le **défaut** pour ne pas perturber une session d'observation nocturne.

### 5.2 Raccourcis et interactions

| Action | Déclencheur |
|--------|-------------|
| Rafraîchir les événements | Bouton 🔄 dans la barre d'outils + menu `Fichier > Rafraîchir` |
| Changer de position active | Sélecteur dans la barre du haut |
| Ouvrir les paramètres | Menu `Edition > Paramètres` |
| Ouvrir le rapport de sync | Icône de statut dans la barre d'état (bas de fenêtre) |

---

## 6. Contraintes et exigences non fonctionnelles

| Exigence | Détail |
|----------|--------|
| **Résilience réseau** | Toute source indisponible ne bloque ni ne crashe l'application |
| **Performance UI** | L'UI reste fluide (≥ 30 fps) pendant les appels réseau |
| **Portabilité** | Compilation sans modification sur Linux et Windows |
| **Stockage minimal** | Pas de base de données embarquée lourde (SQLite non requis initialement) |
| **Accessibilité** | Taille de police configurable (futur) |
| **Sécurité** | Aucune donnée personnelle transmise aux sources externes au-delà de lat/lon |

---

## 7. Roadmap / Jalons

| Jalon | Contenu |
|-------|---------|
| **M0 — Scaffolding** | Projet Rust, structure de fichiers, modèles de données, UI squelette |
| **M1 — Positions** | CRUD positions, sélection active, persistance TOML |
| **M2 — Source ISS** | Intégration passes ISS (Open-Notify ou N2YO), affichage liste |
| **M3 — Autres sources** | Planètes (JPL Horizons), météores (IMO), comètes (MPC) |
| **M4 — Radioastronomie** | Sources ARISS, transit solaire, comètes radio |
| **M5 — Paramétrage** | Thèmes, fréquence de MAJ, rapport de synchronisation complet |
| **M6 — Polish** | Filtres, vue détail enrichie, icônes, packaging |

---

## 8. Questions ouvertes

| # | Question |
|---|----------|
| Q1 | Quel framework UI retenir définitivement (egui vs Slint) ? |
| Q2 | Utiliser un cache SQLite pour de meilleures performances à terme ? |
| Q3 | Ajouter une carte du ciel (vue polaire) en vue future ? |
| Q4 | Internationalisation (FR/EN) dès le départ ou ultérieurement ? |
| Q5 | Packaging : AppImage (Linux), NSIS/MSI (Windows) ? |

---

*Fin du document de spécification — AstroFeed v0.1.0-draft*
