use crate::services::utils::{get_project_root, get_workspace_crates};
use anyhow::{Context, Result};
use fxhash::FxHashMap;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt::Write as FmtWrite;
use std::fs;
use std::path::{Path, PathBuf};

// --- Constants ---
const MANIFEST_PATH: &str = "infra/database/src/generated/migrations_manifest.rs";
const INFRA_DIR: &str = "infra";
const FEATURE_DIR: &str = "crates/features";

// --- Public API ---

/// Generates the compiled migration manifest and permission registry for the workspace.
///
/// This function acts as the build-time compiler for the database schema. It discovers
/// migration scripts across the Vertical Slice Architecture, resolves their dependencies,
/// and produces a static Rust source file containing the execution plan.
///
/// # Returns
/// * `Ok(())` - If the manifest was successfully generated and written to disk.
/// * `Err` - If any step of the discovery, resolution, or I/O fails.
///
/// # Errors
/// This function will return an error if:
/// - The `bootstrap` migration is missing or duplicated.
/// - A circular dependency exists between crates.
/// - A crate depends on a non-existent crate.
/// - File I/O fails (permissions, missing paths).
/// - Migration filenames do not adhere to the `0000-name` format.
pub fn codegen_migrations() -> Result<()> {
    let project_root = get_project_root()?;
    let manifest_path = project_root.join(MANIFEST_PATH);

    // 1. Discovery
    let raw_nodes = discover_nodes()?;

    // 2. Resolution (The heavy lifting)
    let sorted_nodes = resolve_execution_order(raw_nodes)
        .context("Failed to resolve migration dependency graph")?;

    // 3. Codegen
    let manifest_content = render_manifest(&sorted_nodes, &project_root)?;

    // 4. Output
    ensure_parent_dir(&manifest_path)?;
    fs::write(&manifest_path, manifest_content)
        .with_context(|| format!("Failed to write manifest to {}", manifest_path.display()))?;

    println!(
        "✅ Generated migration manifest: {} migrations across {} crates.",
        sorted_nodes.iter().map(|n| n.files.len()).sum::<usize>(),
        sorted_nodes.len()
    );

    Ok(())
}

// --- Domain Models ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NodeKind {
    Infra,
    Feature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MigrationNode {
    key: String,
    name: String,
    description: Option<String>,
    files: Vec<PathBuf>,
    depends_on: Vec<String>,
    permissions: Vec<String>,
    kind: NodeKind,
    is_bootstrap: bool,
}

// --- Logic: Graph Resolution ---

/// Resolves the linear execution order of migrations using a modified Kahn's Algorithm.
///
/// # Algorithm
/// 1. Constructs the graph with explicit dependencies from `Cargo.toml`.
/// 2. Injects implicit dependencies:
///    - **Root Rule**: All nodes depend on `bootstrap` (except `bootstrap` itself).
///    - **Layer Rule**: All `Feature` nodes depend on all `Infra` nodes.
/// 3. Performs a topological sort using a priority queue to ensure deterministic ordering
///    among independent nodes (prioritizing Infra > Feature).
fn resolve_execution_order(nodes: Vec<MigrationNode>) -> Result<Vec<MigrationNode>> {
    let node_map: FxHashMap<String, MigrationNode> =
        nodes.into_iter().map(|n| (n.key.clone(), n)).collect();

    // 1. Identification
    let bootstrap_key = node_map
        .values()
        .find(|n| n.is_bootstrap)
        .map(|n| n.key.clone())
        .ok_or_else(|| anyhow::anyhow!("Missing crate with `bootstrap = true`"))?;

    let infra_keys: Vec<String> =
        node_map.values().filter(|n| n.kind == NodeKind::Infra).map(|n| n.key.clone()).collect();

    // 2. Graph Construction (Adjacency List + In-Degree)
    let mut adj: FxHashMap<String, Vec<String>> = FxHashMap::default();
    let mut in_degree: FxHashMap<String, usize> = node_map.keys().map(|k| (k.clone(), 0)).collect();

    for node in node_map.values() {
        let mut deps = node.depends_on.clone();

        // Apply Implicit Rules
        if node.key != bootstrap_key && !deps.contains(&bootstrap_key) {
            deps.push(bootstrap_key.clone());
        }

        if node.kind == NodeKind::Feature {
            for infra in &infra_keys {
                if !deps.contains(infra) {
                    deps.push(infra.clone());
                }
            }
        }

        // Validate & Build Edges
        for dep in deps {
            if !node_map.contains_key(&dep) {
                return Err(anyhow::anyhow!(
                    "Crate '{}' depends on unknown crate '{dep}'",
                    node.key
                ));
            }
            adj.entry(dep).or_default().push(node.key.clone());
            *in_degree.get_mut(&node.key).unwrap() += 1;
        }
    }

    // 3. Topological Sort
    let mut queue = BinaryHeap::new();
    for (key, &deg) in &in_degree {
        if deg == 0 {
            queue.push(PriorityNode::new(key.clone(), &node_map[key]));
        }
    }

    let mut sorted = Vec::with_capacity(node_map.len());
    while let Some(p_node) = queue.pop() {
        let key = p_node.key;
        sorted.push(node_map[&key].clone());

        if let Some(neighbors) = adj.get(&key) {
            for neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(PriorityNode::new(neighbor.clone(), &node_map[neighbor]));
                }
            }
        }
    }

    if sorted.len() != node_map.len() {
        return Err(anyhow::anyhow!("Circular dependency detected. Graph contains cycles."));
    }

    Ok(sorted)
}

// --- Logic: Utils ---

#[derive(Debug, Eq, PartialEq)]
struct PriorityNode {
    score: u8, // Higher = popped first
    key: String,
}

impl PriorityNode {
    const fn new(key: String, node: &MigrationNode) -> Self {
        let score = match node.kind {
            NodeKind::Infra => 2,
            NodeKind::Feature => 1,
        };
        Self { score, key }
    }
}

impl Ord for PriorityNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score).then_with(|| other.key.cmp(&self.key))
    }
}
impl PartialOrd for PriorityNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn discover_nodes() -> Result<Vec<MigrationNode>> {
    let mut nodes = Vec::new();

    let mut scan = |dir: &str, kind: NodeKind| -> Result<()> {
        for crate_info in get_workspace_crates(dir)? {
            let migrations_dir = crate_info.path.join("migrations");
            if !migrations_dir.exists() {
                continue;
            }

            let files = read_surql_files(&migrations_dir)?;
            if files.is_empty() {
                continue;
            }

            let raw_key = crate_info
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .context("Invalid crate name")?;

            let key =
                if kind == NodeKind::Infra { format!("sys.{raw_key}") } else { raw_key.to_owned() };
            let config = load_migration_config(&crate_info.path)?;

            nodes.push(MigrationNode {
                key,
                name: title_case(raw_key),
                description: crate_info.package.description.clone(),
                files,
                depends_on: config.depends_on,
                permissions: config.permissions,
                kind,
                is_bootstrap: config.bootstrap,
            });
        }
        Ok(())
    };

    scan(INFRA_DIR, NodeKind::Infra)?;
    scan(FEATURE_DIR, NodeKind::Feature)?;

    if nodes.iter().filter(|n| n.is_bootstrap).count() > 1 {
        return Err(anyhow::anyhow!("Multiple bootstrap crates defined"));
    }

    Ok(nodes)
}

// --- View: Rendering ---

fn render_manifest(nodes: &[MigrationNode], root: &Path) -> Result<String> {
    let mut w = String::new();
    writeln!(w, "//! Auto-generated by `cargo xtask codegen migrations`.")?;
    writeln!(w, "//! Do not edit by hand.\n")?;
    writeln!(w, "use crate::migrations::{{Migration, Permissions}};\n")?;

    // 1. Migrations Vector
    writeln!(w, "#[must_use]")?;
    writeln!(w, "pub(crate) fn builtin_migrations() -> Vec<Migration> {{")?;
    writeln!(w, "    vec![")?;
    for node in nodes {
        for file in &node.files {
            render_entry(&mut w, node, file, root)?;
        }
    }
    writeln!(w, "    ]")?;
    writeln!(w, "}}")?;

    // 2. Permission Registry
    writeln!(w, "#[must_use]")?;
    writeln!(w, "pub(crate) fn builtin_registry() -> Vec<Permissions> {{")?;
    writeln!(w, "    vec![")?;

    render_permissions(&mut w, nodes)?;

    writeln!(w, "    ]")?;
    writeln!(w, "}}")?;

    Ok(w)
}

fn render_entry(w: &mut String, node: &MigrationNode, file: &Path, root: &Path) -> Result<()> {
    let version = extract_version(file)?;
    let rel_path = resolve_relative_path(file, root)?;
    let checksum = calculate_checksum(file)?;
    let desc = node
        .description
        .as_deref()
        .map_or_else(|| "None".to_owned(), |d| format!("Some(\"{}\")", escape_str(d)));

    writeln!(w, "        Migration::new(")?;
    writeln!(w, "            \"{}\",", escape_str(&node.key))?;
    writeln!(w, "            \"{}\",", escape_str(&node.name))?;
    writeln!(w, "            {desc},")?;
    writeln!(w, "            \"{}\",", escape_str(&version))?;
    writeln!(w, "            include_str!(\"{rel_path}\"),")?;
    writeln!(w, "            \"{checksum}\",")?;
    writeln!(w, "            {},", node.is_bootstrap)?;
    writeln!(w, "        ),")?;
    Ok(())
}

fn render_permissions(w: &mut String, nodes: &[MigrationNode]) -> Result<()> {
    let mut map: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for node in nodes {
        if !node.permissions.is_empty() {
            let mut perms: Vec<&str> = node.permissions.iter().map(String::as_str).collect();
            perms.sort_unstable();
            map.insert(&node.key, perms);
        }
    }

    for (key, perms) in map {
        writeln!(w, "        Permissions::new(")?;
        writeln!(w, "            \"{}\", ", escape_str(key))?;
        writeln!(w, "            vec![")?;
        for perm in perms {
            writeln!(w, "                \"{}\",", escape_str(perm))?;
        }
        writeln!(w, "            ]),")?;
    }

    Ok(())
}

// --- IO / Helpers ---

#[derive(Deserialize, Default)]
struct PackageMetadata {
    package: Option<PackageConfig>,
}
#[derive(Deserialize, Default)]
struct PackageConfig {
    metadata: Option<MetadataConfig>,
}
#[derive(Deserialize, Default)]
struct MetadataConfig {
    migrations: Option<MigrationConfig>,
}
#[derive(Deserialize, Default)]
struct MigrationConfig {
    #[serde(default)]
    depends_on: Vec<String>,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    bootstrap: bool,
}

fn load_migration_config(path: &Path) -> Result<MigrationConfig> {
    let content = fs::read_to_string(path.join("Cargo.toml"))?;
    let meta: PackageMetadata = toml::from_str(&content)?;
    Ok(meta.package.and_then(|p| p.metadata).and_then(|m| m.migrations).unwrap_or_default())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn read_surql_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "surql") {
            validate_sql_content(&path)?;
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn validate_sql_content(path: &Path) -> Result<()> {
    let s = fs::read_to_string(path)?.to_lowercase();
    if s.contains("begin transaction") || s.contains("commit transaction") {
        return Err(anyhow::anyhow!(
            "Manual transaction control prohibited in '{}'. The migration runner handles transactions.",
            path.display()
        ));
    }
    Ok(())
}

fn extract_version(path: &Path) -> Result<String> {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

    // Expect format: 0000-name
    if !stem.chars().next().map_or(false, |c| c.is_ascii_digit()) || !stem.contains('-') {
        return Err(anyhow::anyhow!("Invalid version format (expected '0000-name'): {stem}"));
    }
    Ok(stem.to_owned())
}

fn resolve_relative_path(path: &Path, root: &Path) -> Result<String> {
    let path_str = path.to_str().unwrap().replace('\\', "/");
    let root_str = root.to_str().unwrap().replace('\\', "/");

    path_str
        .strip_prefix(&format!("{root_str}/"))
        .map(|s| format!("../../../../{s}"))
        .ok_or_else(|| anyhow::anyhow!("Migration file outside project root"))
}

fn calculate_checksum(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hex::encode(hasher.finalize()))
}

// --- String Utilities ---

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn title_case(s: &str) -> String {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut c = part.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().collect::<String>() + c.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(key: &str, kind: NodeKind, bootstrap: bool, deps: &[&str]) -> MigrationNode {
        MigrationNode {
            key: key.into(),
            name: key.into(),
            description: None,
            files: vec![],
            depends_on: deps.iter().map(ToString::to_string).collect(),
            permissions: vec![],
            kind,
            is_bootstrap: bootstrap,
        }
    }

    #[test]
    fn test_topological_sort_happy_path() {
        let nodes = vec![
            node("sys.auth", NodeKind::Infra, false, &[]),
            node("sys.boot", NodeKind::Infra, true, &[]),
            node("billing", NodeKind::Feature, false, &["sys.auth"]),
            node("dashboard", NodeKind::Feature, false, &[]),
        ];

        let sorted = resolve_execution_order(nodes).unwrap();
        let keys: Vec<&str> = sorted.iter().map(|n| n.key.as_str()).collect();

        // 1. Boot must be first
        assert_eq!(keys[0], "sys.boot");

        // 2. Auth (Infra) must come before Features
        let auth_idx = keys.iter().position(|&k| k == "sys.auth").unwrap();
        let bill_idx = keys.iter().position(|&k| k == "billing").unwrap();
        let dash_idx = keys.iter().position(|&k| k == "dashboard").unwrap();

        assert!(auth_idx < bill_idx);
        assert!(auth_idx < dash_idx);
    }

    #[test]
    fn test_circular_dependency_errors() {
        let nodes = vec![
            node("sys.boot", NodeKind::Infra, true, &[]),
            node("feat.a", NodeKind::Feature, false, &["feat.b"]),
            node("feat.b", NodeKind::Feature, false, &["feat.a"]),
        ];

        let result = resolve_execution_order(nodes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circular"));
    }

    #[test]
    fn test_implicit_infra_dependency() {
        let nodes = vec![
            node("sys.boot", NodeKind::Infra, true, &[]),
            node("sys.core", NodeKind::Infra, false, &[]),
            node("feat.a", NodeKind::Feature, false, &[]),
        ];

        let sorted = resolve_execution_order(nodes).unwrap();
        let keys: Vec<&str> = sorted.iter().map(|n| n.key.as_str()).collect();

        assert_eq!(keys, vec!["sys.boot", "sys.core", "feat.a"]);
    }
}
