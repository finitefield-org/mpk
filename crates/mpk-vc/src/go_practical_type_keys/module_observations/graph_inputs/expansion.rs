//! Go 1.25 expandGraph/updatePrunedRoots with no package/add roots. Every
//! admitted main language (Go 1.22–1.25) uses a pruned module graph.
use super::*;

pub(super) struct Phase {
    pub summaries: BTreeMap<GoSourceGraphModule, Summary>,
    pub selected: BTreeMap<String, Vec<u8>>,
}
fn select(selected: &mut BTreeMap<String, Vec<u8>>, m: &GoSourceGraphModule) {
    match selected.get_mut(&m.path) {
        None => {
            selected.insert(m.path.clone(), m.version.clone());
        }
        Some(old) => {
            // The main module's empty version outranks every real version.
            if !old.is_empty()
                && (m.version.is_empty()
                    || manifest::graph_version_compare(&m.path, old, &m.version).is_lt())
            {
                *old = m.version.clone()
            }
        }
    }
}
/// loadModFile resolves redundant/main paths before any graph is loaded.
/// Discarded lower root manifests are not implicit graph inputs.
pub(super) fn bootstrap_roots(
    main: &str,
    roots: &[GoSourceGraphModule],
) -> Vec<GoSourceGraphModule> {
    let mut selected = BTreeMap::from([(main.to_string(), vec![])]);
    for root in roots {
        select(&mut selected, root)
    }
    let mut paths = BTreeSet::from([main.to_string()]);
    roots
        .iter()
        .filter(|m| paths.insert(m.path.clone()))
        .map(|m| GoSourceGraphModule {
            path: m.path.clone(),
            version: selected[&m.path].clone(),
        })
        .collect()
}
pub(super) fn phase(
    main: &str,
    roots: &[GoSourceGraphModule],
    mut summary: impl FnMut(&GoSourceGraphModule) -> GraphResult<Summary>,
) -> GraphResult<Phase> {
    let mut selected = BTreeMap::from([(main.to_string(), vec![])]);
    for root in roots {
        select(&mut selected, root)
    }
    let mut queue = roots.iter().cloned().map(|m| (m, true)).collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    let mut loaded = BTreeMap::new();
    let mut at = 0;
    while at < queue.len() {
        let (m, pruned) = queue[at].clone();
        at += 1;
        if !seen.insert((m.clone(), pruned)) {
            continue;
        }
        if !loaded.contains_key(&m) {
            let row = summary(&m)?;
            // Requirements influence selection even when their manifests lie
            // beyond the pruning horizon and are never read.
            for r in &row.requirements {
                select(&mut selected, r)
            }
            loaded.insert(m.clone(), row);
        }
        let row = &loaded[&m];
        if !pruned || !row.wire.pruned {
            for r in &row.requirements {
                queue.push((r.clone(), pruned && row.wire.pruned));
            }
        }
    }
    Ok(Phase {
        summaries: loaded,
        selected,
    })
}
pub(super) struct Expansion {
    pub roots: Vec<GoSourceGraphModule>,
    pub initial: BTreeMap<GoSourceGraphModule, Summary>,
    pub final_phase: Phase,
    pub manifests: BTreeMap<GoSourceGraphModule, GoSourceGraphManifest>,
}
fn retain(output: &mut BTreeMap<GoSourceGraphModule, GoSourceGraphManifest>, phase: &Phase) {
    for (m, row) in &phase.summaries {
        if let Some(source) = &row.manifest {
            output.insert(m.clone(), source.clone());
        }
    }
}
pub(super) fn expand(
    main: &str,
    mut roots: Vec<GoSourceGraphModule>,
    mut current: Phase,
    mut load: impl FnMut(&[GoSourceGraphModule]) -> GraphResult<Phase>,
) -> GraphResult<Expansion> {
    let initial = current.summaries.clone();
    let mut manifests = BTreeMap::new();
    retain(&mut manifests, &current);
    // expandGraph preserves the original root slice (including duplicates)
    // when all roots are already selected. Deduplication belongs to the
    // updatePrunedRoots path, which is entered only for inconsistent roots.
    while roots
        .iter()
        .any(|m| current.selected.get(&m.path) != Some(&m.version))
    {
        let mut paths = BTreeSet::from([main.to_string()]);
        let next = roots
            .iter()
            .filter(|m| paths.insert(m.path.clone()))
            .map(|m| {
                Ok(GoSourceGraphModule {
                    path: m.path.clone(),
                    version: current.selected.get(&m.path).ok_or(G::Selection)?.clone(),
                })
            })
            .collect::<GraphResult<Vec<_>>>()?;
        ensure(next != roots, G::Selection)?;
        roots = next;
        current = load(&roots)?;
        retain(&mut manifests, &current);
    }
    Ok(Expansion {
        roots,
        initial,
        final_phase: current,
        manifests,
    })
}
