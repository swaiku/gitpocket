## v0.1.0 (12-11-2026)

### ♻️ Refactoring

- Migrate to Tauri, SvelteKit and shadcn-svelte

Replace the Node web server with a Tauri 2 desktop app: Rust backend
calling GitHub CLI, SvelteKit SPA frontend in TypeScript with Tailwind v4
and shadcn-svelte. Switch tooling to Bun, update CI and README, and add a
devcontainer with the Tauri system dependencies and X11 forwarding.

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([3746529](https://github.com/swaiku/gitpocket/commit/37465295655e61e4eaa90baf147cb332b508d951))
- Extract clone and open actions into shared modules

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([726ac2b](https://github.com/swaiku/gitpocket/commit/726ac2be6c069ceb1e1b7ceb62a7104e517cd960))

### ✨ Features

- Add clone and open-with actions, icons and recursive repo detection

Clone repositories with gh into a configurable folder, detect existing
clones recursively, and open them in Claude Desktop, Codex or installed
IDEs. Show app logos and repository icons (local logo or owner avatar).

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([b393156](https://github.com/swaiku/gitpocket/commit/b39315621288ff2cd59612ff7efc7dcb55b07210))
- Add multi-select owner filter with search and clear

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([475d2a1](https://github.com/swaiku/gitpocket/commit/475d2a1292d4bd1bf5b8002c9ac0dce63eb49b38))
- Add a syntax-highlighted file viewer panel

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([7ef11be](https://github.com/swaiku/gitpocket/commit/7ef11be1fe42e1ad4a484a07de59cb6611f74484))
- Add a command runner with streamed output

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([880dac6](https://github.com/swaiku/gitpocket/commit/880dac6aaa7fb2802fc04f0c0e9d613acfb7aa98))
- Add a local copy panel with git status

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([63e4bcc](https://github.com/swaiku/gitpocket/commit/63e4bcceb65b4669cc42c654620b213948f49496))
- Switch local and GitHub branches from searchable menus

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([8885cb5](https://github.com/swaiku/gitpocket/commit/8885cb5a905cb1ce56faf8521efff38a3d595f2c))
- Add a repository page with file browser, README and local tools

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([0a1252d](https://github.com/swaiku/gitpocket/commit/0a1252d4d99a40804ac7eec756a34ff04e7d8e68))

### 💄 UI & Style

- Redesign UI with dark theme, PR counts and card view

Sticky toolbar with tab counts and compact filters, system-following dark
theme, open pull request counts via GraphQL, and a list/cards layout toggle.

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([a599326](https://github.com/swaiku/gitpocket/commit/a5993264e029fe879c8e9b6541cca4f26a6a62ac))
- Show the main language as a colored badge

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([59dec0c](https://github.com/swaiku/gitpocket/commit/59dec0c5b40e2dae8704dc41b79d4578d2fca111))
- Add line numbers to the file viewer

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([e347700](https://github.com/swaiku/gitpocket/commit/e347700f6a5898166cd8d12343ec06215fdb7ece))
- Make the command output panel resizable

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([4311b4e](https://github.com/swaiku/gitpocket/commit/4311b4e88059d0ac825aa63bdd9d7f0e3dac1377))
- Add per-type file and folder icons

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([bbad7d1](https://github.com/swaiku/gitpocket/commit/bbad7d15df29737322a8f04f68b9e6719ccfb11e))

### 📝 Documentation

- Rewrite the README with a banner and shieldcn badges

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([f9c29eb](https://github.com/swaiku/gitpocket/commit/f9c29eb6f9093858a7d4fe81f3672ebd57f39399))

### 🔀 Other

- Repocket with per-user GitHub sign-in ([5dbdb2a](https://github.com/swaiku/gitpocket/commit/5dbdb2a7fcbba84794a288683e37f8b6cfab05e1))
- installation, GitHub setup and troubleshooting ([c0886e5](https://github.com/swaiku/gitpocket/commit/c0886e5ba25110083f26456c472eef45db52a28d))
- setup guide on local use ([9a8d363](https://github.com/swaiku/gitpocket/commit/9a8d3634b773213d9658e43f5ee128d35f2f743c))
- README to a step-by-step local installation ([988ba26](https://github.com/swaiku/gitpocket/commit/988ba26fd93c92ed7e7bc21eb1957364e3fb707a))
- the project purpose in the README ([a8cad7f](https://github.com/swaiku/gitpocket/commit/a8cad7ff30607818c5bbea72f98daac673f58d02))
- Repocket for public use and contributions ([8b09428](https://github.com/swaiku/gitpocket/commit/8b09428eb7390e4293f315396e77d4952ca6e4ef))
- project to GitPocket ([b1219dd](https://github.com/swaiku/gitpocket/commit/b1219ddf69ab7120e376b428afffb7d399c82d45))
- Translate code and interface to English

Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com> ([505b388](https://github.com/swaiku/gitpocket/commit/505b388e2d4bb85932a1e043a53bc2f9fadb124b))

