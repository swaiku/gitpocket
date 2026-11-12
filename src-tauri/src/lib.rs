use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;

#[derive(Deserialize, Clone)]
struct Raw {
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
}

#[derive(Deserialize)]
struct Invitation {
    created_at: String,
    repository: Raw,
}

#[derive(Serialize)]
struct Overview {
    login: String,
    repos: Vec<Repo>,
    invitations: Vec<Repo>,
    warning: Option<String>,
}

// Les tokens restent gérés par GitHub CLI : l'app n'en stocke aucun.
fn gh<T: for<'a> Deserialize<'a>>(endpoint: &str, paginate: bool) -> Result<T, String> {
    let mut cmd = Command::new("gh");
    cmd.args(["api", "--hostname", "github.com", endpoint]);
    if paginate {
        cmd.args(["--paginate", "--slurp"]);
    }
    let out = cmd.output().map_err(|_| "GitHub CLI (gh) est introuvable. Installe-le puis lance « gh auth login --hostname github.com ».".to_string())?;
    if !out.status.success() {
        return Err("Impossible de charger GitHub. Vérifie ta connexion Internet et lance « gh auth login --hostname github.com ». Vérifie aussi les autorisations et le SSO de ton organisation.".into());
    }
    let value: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let value = match (paginate, value) {
        // --slurp renvoie un tableau de pages : on l'aplatit.
        (true, serde_json::Value::Array(pages)) => serde_json::Value::Array(
            pages.into_iter().flat_map(|p| match p { serde_json::Value::Array(a) => a, o => vec![o] }).collect(),
        ),
        (_, v) => v,
    };
    serde_json::from_value(value).map_err(|e| e.to_string())
}

fn repo(r: Raw, collaborator: bool, invited_at: Option<String>) -> Repo {
    Repo { id: r.id, full_name: r.full_name, description: r.description, private: r.private, language: r.language, updated_at: r.updated_at, pushed_at: r.pushed_at, archived: r.archived, collaborator, invited_at }
}

#[tauri::command]
async fn overview() -> Result<Overview, String> {
    tauri::async_runtime::spawn_blocking(|| {
        #[derive(Deserialize)]
        struct User { login: String }
        let user: User = gh("user", false)?;
        let repos: Vec<Raw> = gh("user/repos?affiliation=owner,collaborator,organization_member&sort=updated&per_page=100", true)?;
        let collabs: Vec<Raw> = gh("user/repos?affiliation=collaborator&per_page=100", true)?;
        let collab_ids: HashSet<u64> = collabs.iter().map(|r| r.id).collect();
        let (invitations, warning) = match gh::<Vec<Invitation>>("user/repository_invitations?per_page=100", true) {
            Ok(v) => (v, None),
            Err(_) => (vec![], Some("Les invitations n’ont pas pu être chargées. Vérifie les permissions GitHub puis réessaie.".to_string())),
        };
        Ok(Overview {
            login: user.login,
            repos: repos.into_iter().map(|r| { let c = collab_ids.contains(&r.id); repo(r, c, None) }).collect(),
            invitations: invitations.into_iter().map(|i| repo(i.repository, false, Some(i.created_at))).collect(),
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
        .invoke_handler(tauri::generate_handler![overview])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
