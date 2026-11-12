# GitPocket ↗

GitPocket rassemble dans une seule page les dépôts GitHub auxquels tu as accès, y compris ceux où tu es invité comme collaborateur, pour les retrouver facilement et voir lesquels ont reçu du code récemment.

## Stack

Tauri 2 (Rust) · SvelteKit (SPA) · TypeScript · Tailwind v4 · shadcn-svelte.

Le backend Rust appelle GitHub CLI (`gh api`) : l'app ne stocke aucun jeton.

## Développement

Prérequis : [Bun](https://bun.sh), [Rust](https://rustup.rs), les [dépendances système de Tauri](https://tauri.app/start/prerequisites/) et [GitHub CLI](https://cli.github.com/) connecté (`gh auth login --hostname github.com`).

```sh
bun install
bun run tauri dev
```

### Devcontainer

`.devcontainer/` fournit Rust, Bun, GitHub CLI et les dépendances système de Tauri. Il permet de lancer `bun run check`, `bun test src/lib` et `cargo check` (dans `src-tauri/`). Pour afficher la fenêtre de l'app (`tauri dev`), le conteneur partage le serveur X11 de l'hôte (Xwayland suffit). Sur l'hôte, autorise une fois les connexions locales avant d'ouvrir le conteneur : `xhost +local:`. `gh auth login` doit être refait dans le conteneur.

Tests et vérification des types : `bun test src/lib` et `bun run check`.
