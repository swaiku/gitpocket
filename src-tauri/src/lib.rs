use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Manager};

#[derive(Deserialize, Clone, Default)]
struct Owner {
    avatar_url: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Raw {
    #[serde(default)]
    owner: Owner,
    id: u64,
    full_name: String,
    description: Option<String>,
    private: bool,
    language: Option<String>,
    updated_at: Option<String>,
    pushed_at: Option<String>,
    archived: bool,
}

#[derive(Serialize)]
struct Repo {
    id: u64,
    full_name: String,
    description: Option<String>,
    private: bool,
    language: Option<String>,
    updated_at: Option<String>,
    pushed_at: Option<String>,
    archived: bool,
    collaborator: bool,
    invited_at: Option<String>,
    open_prs: Option<u64>,
    local_path: Option<String>,
    avatar_url: Option<String>,
    icon: Option<String>,
}

#[derive(Deserialize)]
struct Invitation {
    created_at: String,
    repository: Raw,
}

#[derive(Serialize)]
struct Overview {
    login: String,
    clone_dir: String,
    repos: Vec<Repo>,
    invitations: Vec<Repo>,
    warning: Option<String>,
}

// Tokens stay managed by GitHub CLI: the app stores none.
fn gh_json(args: &[&str], paginate: bool) -> Result<serde_json::Value, String> {
    let mut cmd = Command::new("gh");
    cmd.args(["api", "--hostname", "github.com"]).args(args);
    if paginate {
        cmd.args(["--paginate", "--slurp"]);
    }
    let out = cmd.output().map_err(|_| "GitHub CLI (gh) not found. Install it, then run `gh auth login --hostname github.com`.".to_string())?;
    if !out.status.success() {
        return Err("Could not load GitHub. Check your Internet connection and run `gh auth login --hostname github.com`. Also check your organization's permissions and SSO.".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    Ok(match (paginate, value) {
        // --slurp returns an array of pages: flatten it.
        (true, serde_json::Value::Array(pages)) => serde_json::Value::Array(
            pages.into_iter().flat_map(|p| match p { serde_json::Value::Array(a) => a, o => vec![o] }).collect(),
        ),
        (_, v) => v,
    })
}

fn gh<T: for<'a> Deserialize<'a>>(endpoint: &str, paginate: bool) -> Result<T, String> {
    serde_json::from_value(gh_json(&[endpoint], paginate)?).map_err(|e| e.to_string())
}

// Open pull request counts are only available through GraphQL.
fn open_prs() -> Result<HashMap<String, u64>, String> {
    let query = "query($endCursor: String) { viewer { repositories(first: 100, after: $endCursor, affiliations: [OWNER, COLLABORATOR, ORGANIZATION_MEMBER]) { pageInfo { hasNextPage endCursor } nodes { nameWithOwner pullRequests(states: OPEN) { totalCount } } } } }";
    let pages = gh_json(&["graphql", "-f", &format!("query={query}")], true)?;
    let nodes = pages.as_array().into_iter().flatten().filter_map(|p| p["data"]["viewer"]["repositories"]["nodes"].as_array()).flatten();
    Ok(nodes.filter_map(|n| Some((n["nameWithOwner"].as_str()?.to_lowercase(), n["pullRequests"]["totalCount"].as_u64()?))).collect())
}

fn repo(r: Raw, collaborator: bool, invited_at: Option<String>, open_prs: Option<u64>) -> Repo {
    let avatar_url = r.owner.avatar_url.clone();
    Repo { id: r.id, full_name: r.full_name, description: r.description, private: r.private, language: r.language, updated_at: r.updated_at, pushed_at: r.pushed_at, archived: r.archived, collaborator, invited_at, open_prs, local_path: None, avatar_url, icon: None }
}

#[derive(Serialize, Deserialize, Default)]
struct Settings {
    clone_dir: Option<PathBuf>,
}

fn settings_file(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app.path().app_config_dir().map_err(|e| e.to_string())?.join("settings.json"))
}

fn clone_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let saved = std::fs::read(settings_file(app)?).ok().and_then(|b| serde_json::from_slice::<Settings>(&b).ok()).and_then(|s| s.clone_dir);
    match saved {
        Some(dir) => Ok(dir),
        None => Ok(app.path().home_dir().map_err(|e| e.to_string())?.join("Projects")),
    }
}

#[tauri::command]
fn set_clone_dir(app: AppHandle, dir: String) -> Result<(), String> {
    let file = settings_file(&app)?;
    std::fs::create_dir_all(file.parent().unwrap()).map_err(|e| e.to_string())?;
    let json = serde_json::to_vec(&Settings { clone_dir: Some(dir.into()) }).map_err(|e| e.to_string())?;
    std::fs::write(file, json).map_err(|e| e.to_string())
}

// Maps lowercase "owner/name" to the local folder, by reading each repository's git origin.
// A folder with a .git stops the descent (nested repos are ignored); depth is capped to keep the scan fast.
fn find_local(dir: &Path) -> HashMap<String, String> {
    fn walk(dir: &Path, depth: u8, found: &mut HashMap<String, String>) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if !path.is_dir() || name.starts_with('.') || ["node_modules", "target"].contains(&&*name) {
                continue;
            }
            match std::fs::read_to_string(path.join(".git/config")) {
                Ok(config) => {
                    for line in config.lines() {
                        let Some((key, url)) = line.split_once('=') else { continue };
                        if key.trim() != "url" { continue }
                        if let Some((_, rest)) = url.trim().split_once("github.com") {
                            let repo = rest.trim_start_matches([':', '/']).trim_end_matches('/').trim_end_matches(".git");
                            found.entry(repo.to_lowercase()).or_insert_with(|| path.to_string_lossy().into_owned());
                        }
                    }
                }
                Err(_) if depth > 1 && !path.join(".git").exists() => walk(&path, depth - 1, found),
                Err(_) => {}
            }
        }
    }
    let mut found = HashMap::new();
    walk(dir, 4, &mut found);
    found
}

// Looks for a logo in a cloned repository and returns it as a data URL.
// shortcut: fixed list of common locations, upgrade to reading the manifest (package.json, Cargo.toml) if misses are common.
fn find_icon(repo: &Path) -> Option<String> {
    use base64::{Engine, engine::general_purpose::STANDARD};
    const DIRS: &[&str] = &["", "assets", "public", "static", "docs", ".github", "src-tauri/icons", "app/src/main/res/mipmap-xxxhdpi"];
    const FILES: &[&str] = &["icon.svg", "logo.svg", "favicon.svg", "icon.png", "logo.png", "128x128.png", "favicon.png", "ic_launcher.png", "favicon.ico"];
    for dir in DIRS {
        for file in FILES {
            let path = repo.join(dir).join(file);
            let Ok(meta) = std::fs::metadata(&path) else { continue };
            if !meta.is_file() || meta.len() > 256 * 1024 { continue }
            let mime = match path.extension()?.to_str()? {
                "svg" => "image/svg+xml",
                "png" => "image/png",
                _ => "image/x-icon",
            };
            if let Ok(bytes) = std::fs::read(&path) {
                return Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)));
            }
        }
    }
    None
}

#[tauri::command]
fn repo_icon(path: String) -> Option<String> {
    find_icon(Path::new(&path))
}

fn split_repo(full_name: &str) -> Result<(&str, &str), String> {
    let valid = |part: &str| !part.is_empty() && part.chars().any(|c| c != '.') && part.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c));
    match full_name.split_once('/') {
        Some((owner, name)) if valid(owner) && valid(name) => Ok((owner, name)),
        _ => Err("Invalid repository name.".into()),
    }
}

// Builds the `?ref=` query for a branch name, rejecting anything that could alter the request.
fn ref_query(branch: &Option<String>) -> Result<String, String> {
    match branch.as_deref() {
        None | Some("") => Ok(String::new()),
        Some(b) if b.starts_with('-') || b.contains("..") || b.chars().any(|c| c.is_control() || c.is_whitespace() || "?#&~^:\\".contains(c)) => Err("Invalid branch name.".into()),
        Some(b) => Ok(format!("?ref={}", encode_path(b))),
    }
}

fn encode_path(path: &str) -> String {
    path.split('/').map(|seg| seg.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect::<String>()).collect::<Vec<_>>().join("/")
}

#[derive(Serialize)]
struct FileContent {
    // "text" | "image" | "binary" | "large"
    kind: &'static str,
    text: Option<String>,
    data_url: Option<String>,
    size: u64,
}

#[tauri::command]
async fn repo_file(full_name: String, path: String, branch: Option<String>) -> Result<FileContent, String> {
    split_repo(&full_name)?;
    let query = ref_query(&branch)?;
    if path.is_empty() || path.split('/').any(|seg| seg == "..") {
        return Err("Invalid path.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        use base64::{Engine, engine::general_purpose::STANDARD};
        #[derive(Deserialize)]
        struct Raw { size: u64, #[serde(default)] content: String, #[serde(default)] encoding: String }
        let raw: Raw = gh(&format!("repos/{full_name}/contents/{}{query}", encode_path(&path)), false)?;
        // The contents API omits the body above 1 MB.
        if raw.encoding != "base64" || (raw.content.is_empty() && raw.size > 0) {
            return Ok(FileContent { kind: "large", text: None, data_url: None, size: raw.size });
        }
        let compact: String = raw.content.chars().filter(|c| !c.is_whitespace()).collect();
        let bytes = STANDARD.decode(compact).map_err(|e| e.to_string())?;
        let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
        let image = match ext.as_str() { "png" => Some("image/png"), "jpg" | "jpeg" => Some("image/jpeg"), "gif" => Some("image/gif"), "webp" => Some("image/webp"), "svg" => Some("image/svg+xml"), "ico" => Some("image/x-icon"), _ => None };
        if let Some(mime) = image {
            return Ok(FileContent { kind: "image", text: None, data_url: Some(format!("data:{mime};base64,{}", STANDARD.encode(&bytes))), size: raw.size });
        }
        if bytes.contains(&0) {
            return Ok(FileContent { kind: "binary", text: None, data_url: None, size: raw.size });
        }
        Ok(FileContent { kind: "text", text: Some(String::from_utf8_lossy(&bytes).into_owned()), data_url: None, size: raw.size })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn clone_repo(app: AppHandle, full_name: String) -> Result<String, String> {
    let (_, name) = split_repo(&full_name)?;
    let name = name.to_string();
    let dir = clone_dir(&app)?;
    let dest = dir.join(&name);
    if dest.exists() {
        return Err(format!("A folder named “{name}” already exists in {}.", dir.display()));
    }
    tauri::async_runtime::spawn_blocking(move || {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let out = Command::new("gh").args(["repo", "clone", &full_name]).arg(&dest).output().map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(dest.to_string_lossy().into_owned())
        } else {
            Err(format!("Clone failed: {}", String::from_utf8_lossy(&out.stderr).trim()))
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// Launchers: (id, name, binary, leading args). The frontend only sends an id, never a command.
const COMMANDS: &[(&str, &str, &str, &[&str])] = &[
    ("code", "VS Code", "code", &[]),
    ("cursor", "Cursor", "cursor", &[]),
    ("zed", "Zed", "zed", &[]),
    ("codium", "VSCodium", "codium", &[]),
    ("idea", "IntelliJ IDEA", "idea", &[]),
    ("webstorm", "WebStorm", "webstorm", &[]),
    ("pycharm", "PyCharm", "pycharm", &[]),
    ("rustrover", "RustRover", "rustrover", &[]),
    ("subl", "Sublime Text", "subl", &[]),
    ("codex", "Codex", "codex", &["app"]),
];

fn on_path(bin: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(bin).is_file()))
}

fn claude_installed() -> bool {
    if cfg!(target_os = "linux") {
        Command::new("xdg-mime").args(["query", "default", "x-scheme-handler/claude"]).output().is_ok_and(|o| !o.stdout.trim_ascii().is_empty())
    } else if cfg!(target_os = "macos") {
        Path::new("/Applications/Claude.app").exists()
    } else {
        true
    }
}

#[derive(Serialize)]
struct Tool {
    id: &'static str,
    name: &'static str,
    // "url": opened as a deep link by the frontend; "command": launched by open_with.
    kind: &'static str,
}

#[tauri::command]
fn tools() -> Vec<Tool> {
    let mut list = vec![];
    if claude_installed() {
        list.push(Tool { id: "claude", name: "Claude Desktop", kind: "url" });
    }
    list.extend(COMMANDS.iter().filter(|c| on_path(c.2)).map(|c| Tool { id: c.0, name: c.1, kind: "command" }));
    list
}

#[tauri::command]
fn open_with(id: String, path: String) -> Result<(), String> {
    let (_, name, bin, lead) = COMMANDS.iter().find(|c| c.0 == id).ok_or("Unknown application.")?;
    if !Path::new(&path).is_dir() {
        return Err("This folder no longer exists.".into());
    }
    Command::new(bin).args(*lead).arg(&path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().map_err(|e| format!("Could not start {name}: {e}"))?;
    Ok(())
}

#[tauri::command]
async fn overview(app: AppHandle) -> Result<Overview, String> {
    let dir = clone_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let local = find_local(&dir);
        #[derive(Deserialize)]
        struct User { login: String }
        let user: User = gh("user", false)?;
        let repos: Vec<Raw> = gh("user/repos?affiliation=owner,collaborator,organization_member&sort=updated&per_page=100", true)?;
        let collabs: Vec<Raw> = gh("user/repos?affiliation=collaborator&per_page=100", true)?;
        let collab_ids: HashSet<u64> = collabs.iter().map(|r| r.id).collect();
        // Counts are optional: on failure the UI simply shows no number.
        let prs = open_prs().unwrap_or_default();
        let (invitations, warning) = match gh::<Vec<Invitation>>("user/repository_invitations?per_page=100", true) {
            Ok(v) => (v, None),
            Err(_) => (vec![], Some("Invitations could not be loaded. Check your GitHub permissions and try again.".to_string())),
        };
        Ok(Overview {
            login: user.login,
            clone_dir: dir.to_string_lossy().into_owned(),
            repos: repos.into_iter().map(|r| { let c = collab_ids.contains(&r.id); let n = prs.get(&r.full_name.to_lowercase()).copied(); let mut x = repo(r, c, None, n); x.local_path = local.get(&x.full_name.to_lowercase()).cloned(); x.icon = x.local_path.as_deref().and_then(|p| find_icon(Path::new(p))); x }).collect(),
            invitations: invitations.into_iter().map(|i| repo(i.repository, false, Some(i.created_at), None)).collect(),
            warning,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![overview, repo_file, repo_icon, set_clone_dir, clone_repo, tools, open_with])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
