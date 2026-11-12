# GitPocket ↗

GitPocket gathers every GitHub repository you can access in a single window, including the ones where you were invited as a collaborator, so you can find them easily and see which ones recently received code.

## Stack

Tauri 2 (Rust) · SvelteKit (SPA) · TypeScript · Tailwind v4 · shadcn-svelte.

The Rust backend calls GitHub CLI (`gh api`): the app stores no token.

## Development

Requirements: [Bun](https://bun.sh), [Rust](https://rustup.rs), the [Tauri system dependencies](https://tauri.app/start/prerequisites/) and a signed-in [GitHub CLI](https://cli.github.com/) (`gh auth login --hostname github.com`).

```sh
bun install
bun run tauri dev
```

### Devcontainer

`.devcontainer/` provides Rust, Bun, GitHub CLI and the Tauri system dependencies. It lets you run `bun run check`, `bun test src/lib` and `cargo check` (in `src-tauri/`). To display the app window (`tauri dev`), the container shares the host's X11 server (Xwayland is enough). On the host, allow local connections once before opening the container: `xhost +local:`. You must run `gh auth login` again inside the container.

### Clone and open

Each repository has a **Clone** button (via `gh repo clone`) that clones into the folder shown in the header (default `~/Projects`, click to change). Already-cloned repositories are detected by scanning that folder recursively (up to 4 levels deep), and get an **Open** menu listing the installed apps: Claude Desktop (`claude://` deep link), Codex (`codex app`), and IDEs found on your `PATH` (VS Code, Cursor, Zed, VSCodium, JetBrains, Sublime Text). The devcontainer mounts your host `~/Projects` so detection and cloning work there, but host apps are not visible from inside it: run the app on the host to launch them.

Tests and type check: `bun test src/lib` and `bun run check`.
