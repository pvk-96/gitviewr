# GitViewr

Understand your Git repositories.

GitViewr is a desktop application—built with **Tauri 2** (Rust backend + web frontend)—that analyzes a Git repository and turns its history into clear, useful insights: commit timelines, file change matrices, contributor activity, and time-based analytics. It works with local repositories and public GitHub URLs.

> #### Note: GitViewr is a **read-only analyzer**. It never modifies your repository's history, refs, or working tree.

## Features

- **Open a local repository** — pick any folder containing a `.git` directory.
- **Analyze a GitHub repository** — paste a URL like `https://github.com/owner/repository`; GitViewr clones it into a managed cache and analyzes it.
- **Repository overview** — branches, tracked files, contributor count, first/latest commit, total additions/deletions.
- **Commits** — full history (newest first) with per-commit subject, author, timing, and per-file change summaries.
- **Changes** — per-file statistics (times changed, additions, deletions, last touched) sorted by churn.
- **Analytics**
  - Commits over time (monthly buckets).
  - Contributor table with per-author commit/add/delete totals.
  - File hot-spots (files with the most changes).
  - Timeline chart combining commits, additions, and deletions per month.
- **Recent repositories** — the last three distinct recent repositories are remembered (deduplicated, most-recent first).
- **Export** — save a report as **HTML**, **JSON**, or **PDF** to any location.
- **About** — developer site and support links.

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (rustc 1.77+)
- [Node.js](https://nodejs.org) (npm, Vite 5)
- [Tauri 2 prerequisites](https://tauri.app/start/prerequisites/):
  - Linux: `webkit2gtk-4.1`, `gtk3`, `libsoup3`, `libayatana-appindicator3-dev` (and `libssl-dev`, `cmake`, `pkg-config` for OpenSSL/libgit2).
  - macOS: Xcode command-line tools.
  - Windows: WebView2 (preinstalled on Windows 11 / recent Windows 10).

GitViewr uses **libgit2** (via the `git2` crate) for all repository access, so no system `git` binary is required at runtime.

## Getting started

```bash
npm install
```

### Run in development

```bash
npm run tauri dev
```

This starts the Vite dev server (`http://localhost:1420`) and launches the Tauri app.

### Run the tests

```bash
npm run test          # frontend unit tests (Vitest)
cargo test            # Rust unit + integration tests (Rust)
```

### Production build

```bash
npm run tauri build
```

Output installers land in `src-tauri/target/release/bundle/`.

## Project structure

```
.
├── src/                        # Frontend (React)
│   ├── App.jsx                 # App shell, navigation, shared state
│   ├── pages/                  # One page per sidebar tab
│   │   ├── RepositoryPage.jsx  # Open local repo / GitHub URL
│   │   ├── CommitsPage.jsx     # Commit history + diff details
│   │   ├── ChangesPage.jsx     # Per-file change matrix
│   │   ├── AnalyticsPage.jsx   # Charts + contributor/activity tables
│   │   ├── RecentPage.jsx      # Recent repository list (only 3 recent repositories allowed right now for simplicity.)
│   │   ├── ExportPage.jsx      # HTML/JSON/PDF export (plain export onely, formatted export_repot for easy readability will be added later.)
│   │   └── AboutPage.jsx
│   ├── components/             # Sidebar, empty state, commit modal
│   ├── hooks/                  # Analysis progress listener
│   └── utils/                  # Icons, date/number formatting
└── src-tauri/                  # Backend (Rust)
    ├── src/
    │   ├── models/             # AnalysisData, Repository, Commit, …
    │   ├── git/                # RepoInfo, commit-history walker, errors
    │   ├── services/           # Analyzer, GitHub clone, exporters
    │   │   └── exporter/       # HTML / JSON / PDF report writers (will add formatted export_report later)
    │   ├── storage/            # Recent-repository store
    │   ├── commands/           # Tauri IPC commands
    │   └── utils/              # Date conversions
    ├── tests/                  # Integration tests against real git repos
    └── tauri.conf.json
```

## Architecture and data flow

1. The frontend asks the backend to analyze a repository: `analyze_local_repository` (path) or `analyze_github_repository` (URL).
2. The backend opens the repository with libgit2, walks its full history (parents-first diffs with rename detection enabled), and aggregates:
   - per-commit records (subject, author, timestamp, per-file deltas),
   - repository statistics,
   - contributor totals,
   - monthly time buckets,
   - per-file churn statistics.
3. Progress is streamed back via `analysis-progress` events while walking.
4. The frontend renders the result; on completion the backend remembers the repository in the recent store.
5. Exports are produced by writing HTML (inline CSS), JSON (analysis payload + generation metadata), or a hand-rolled PFD document. The JSON analysis payload is passed back from the frontend to the exporter so the report matches exactly what is on screen.

### IPC surface

| Command                   | Purpose                                   |
| ------------------------- | ----------------------------------------- |
| `analyze_local_repository`   | Analyze a local path                      |
| `analyze_github_repository`  | Clone + analyze a GitHub URL              |
| `get_recent_repositories`    | List recent repos                         |
| `export_report`              | Write HTML/JSON/PDF report                |
| `open_external_url`          | Open a URL in the default browser         |

### Storage

- Recent repositories: `{app data dir}/recent.json` (max 3, duplicates not shown)
- GitHub clones: `{app data dir}/repositories/github_{owner}__{repo}` (reused if present).

## Limitations

- Analyzes the **default / checked-out branch** of the repository.
- GitHub support requires a public repository and an internet connection on first analysis.
- PDF export uses a minimal built-in PDF writer (base-14 Helvetica fonts); it is plain-text only, so non-Latin characters render as `?`.
- Large repositories take longer; progress is reported on the Commits/Analytics phases only.
- Merge commits include the aggregate diff against the first parent.

## Future ideas

- Selectable branch / commit range analysis.
- Windows installer (the build currently targets Linux).

## License / Support

See the About tab inside the app. Development site: https://pvk96.in
