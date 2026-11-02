//! Local live-root ownership observer; no CLI or launcher authority.
#[cfg(target_os = "linux")]
#[path = "../src/frontend_registry.rs"]
#[allow(unused_imports)]
mod frontend_registry;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_capture_binding.rs"]
#[allow(dead_code)]
mod go_practical_capture_binding;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_entries.rs"]
#[allow(dead_code)]
mod go_practical_checked_entries;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_installed_compiler.rs"]
#[allow(dead_code)]
mod go_practical_installed_compiler;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_source_roots.rs"]
#[allow(dead_code)]
#[cfg_attr(test, allow(unused_imports))]
mod go_practical_source_roots;

#[cfg(target_os = "linux")]
fn main() {
    use go_practical_capture_binding::GoCaptureSourceRoots;
    use mpk_vc::go_practical_type_keys::*;
    use mpk_vc::release_bundle_v1::{validate_successor_release_registry, GO_TOOLCHAIN_BUNDLE_ID};
    use mpk_vc::semantic_profile_registry::{validate_semantic_profile_registry, RegistryRevision};
    use serde_json::{json, Value};
    use std::error::Error;
    use std::fs::{self, OpenOptions};
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, Instant};

    type Result<T> = std::result::Result<T, Box<dyn Error>>;
    fn fail(message: impl ToString) -> io::Error {
        io::Error::other(message.to_string())
    }
    fn wait(path: &Path) -> Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(30 * 60);
        while Instant::now() < deadline {
            match fs::read(path) {
                Ok(raw) => return Ok(serde_json::from_slice(&raw)?),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    thread::sleep(Duration::from_millis(200));
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(fail("original live-root signal did not arrive").into())
    }
    fn publish(dir: &Path, name: &str, data: &Value) -> Result<()> {
        let temporary = dir.join(format!(".combined-observer-{}-{name}", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(data)?)?;
        file.write_all(b"\n")?;
        drop(file);
        fs::hard_link(&temporary, dir.join(name))?;
        fs::remove_file(temporary)?;
        Ok(())
    }
    fn unhex(text: &str) -> Result<Vec<u8>> {
        let pairs = text.as_bytes().chunks_exact(2);
        if !pairs.remainder().is_empty() {
            return Err(fail("odd hexadecimal input").into());
        }
        pairs
            .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
            .collect()
    }
    fn buffers(packet: &Value, field: &str, directory: &Path) -> Result<Vec<GoTypeSourceBuffer>> {
        let rows = packet[field]
            .as_array()
            .ok_or_else(|| fail("missing source buffers"))?;
        rows.iter()
            .map(|row| {
                let raw = if let Some(text) = row["raw_hex"].as_str() {
                    unhex(text)?
                } else {
                    let name = row["raw_file"]
                        .as_str()
                        .ok_or_else(|| fail("missing original source file"))?;
                    let bytes = name.as_bytes();
                    if bytes.len() != 68
                        || !bytes.ends_with(b".raw")
                        || !bytes[..64].iter().all(u8::is_ascii_hexdigit)
                    {
                        return Err(fail("invalid original source filename").into());
                    }
                    fs::read(directory.join("original-source-objects").join(name))?
                };
                Ok(GoTypeSourceBuffer::import(
                    &serde_json::to_vec(&row["object"])?,
                    &raw,
                )?)
            })
            .collect()
    }
    fn snapshot(packet: &Value, directory: &Path) -> Result<GoSourceSnapshotInputs> {
        Ok(import_go_source_snapshot_inputs(
            &unhex(
                packet["snapshot_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing snapshot"))?,
            )?,
            &buffers(packet, "snapshot_sources", directory)?,
        )?)
    }
    fn exports(packet: &Value, directory: &Path) -> Result<GoSourceExportInputs> {
        let selection = import_go_source_selection_inputs(
            &unhex(
                packet["selection_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing selection"))?,
            )?,
            &unhex(
                packet["envelope_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing envelope"))?,
            )?,
            &unhex(
                packet["components_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing components"))?,
            )?,
            &buffers(packet, "sources", directory)?,
            &buffers(packet, "overlays", directory)?,
        )?;
        let capture = import_go_source_capture_inputs(
            &unhex(
                packet["capture_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing capture"))?,
            )?,
            selection,
            snapshot(packet, directory)?,
        )?;
        Ok(import_go_source_export_inputs(
            &unhex(
                packet["export_inputs_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing export inputs"))?,
            )?,
            capture,
        )?)
    }
    fn candidate_request(
        packet: &Value,
        field: &str,
    ) -> Result<mpk_vc::go_practical_context::GoPrivateCandidateRequest> {
        let raw = unhex(
            packet[field]
                .as_str()
                .ok_or_else(|| fail("missing original Go candidate request"))?,
        )?;
        let value = mpk_vc::parse_strict_json(
            &raw,
            mpk_vc::StrictJsonLimits::new(268_435_456, 268_435_456, 64, 16_777_216),
        )?;
        Ok(
            mpk_vc::go_practical_context::GoPrivateCandidateRegistry::new()?
                .import_request(&value)?,
        )
    }
    fn module_graph(
        packet: &Value,
        exports: GoSourceExportInputs,
    ) -> Result<GoSourceModuleGraphInputs> {
        let observations = import_go_source_module_observations(
            &unhex(
                packet["module_observations_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing original module observations"))?,
            )?,
            exports,
        )?;
        let data = observations.verify_offline_module_data()?;
        Ok(import_go_source_module_graph_inputs(
            &unhex(
                packet["module_graph_inputs_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing original module graph"))?,
            )?,
            data,
        )?)
    }
    fn compiler_inputs(
        packet: &Value,
        directory: &Path,
        original_exports: GoSourceExportInputs,
    ) -> Result<GoSourceCompilerInputs> {
        let semantic = validate_semantic_profile_registry(
            include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
            RegistryRevision::Revision4,
        )?;
        let registry = validate_successor_release_registry(
            include_bytes!("../../../release/bundles/bundle-registry.json"),
            &semantic,
        )?;
        let inventory = &registry
            .registry()
            .toolchain_bundles
            .iter()
            .find(|bundle| bundle.bundle_id == GO_TOOLCHAIN_BUNDLE_ID)
            .ok_or_else(|| fail("registered Go SDK is missing"))?
            .inventory;
        let rows = packet["compiler_files"]
            .as_array()
            .ok_or_else(|| fail("missing original compiler files"))?;
        if rows.len() != inventory.files.len() {
            return Err(fail("registered SDK file count differs").into());
        }
        let mut files = Vec::with_capacity(rows.len());
        for (row, entry) in rows.iter().zip(&inventory.files) {
            let name = format!("{}.raw", entry.sha256);
            if row["entry"] != serde_json::to_value(entry)? || row["raw_file"] != name {
                return Err(fail("original compiler inventory differs").into());
            }
            let raw = fs::read(directory.join("original-compiler-files").join(name))?;
            files.push(GoCompilerFileBuffer::import(
                &entry.path,
                entry.executable,
                &raw,
            )?);
        }
        let record = unhex(
            packet["compiler_inputs_hex"]
                .as_str()
                .ok_or_else(|| fail("missing original compiler inputs"))?,
        )?;
        let owner = import_go_source_compiler_inputs(&record, original_exports, files)?;
        if owner.inventory() != inventory {
            return Err(fail("imported SDK inventory differs from the registered bundle").into());
        }
        Ok(owner)
    }
    // This constructor is compiled only for the local test observer. Its
    // complete SDK bytes come from the independently imported original image;
    // it does not open or authorize a production installed release.
    #[cfg(test)]
    fn test_installed_owner(
        compiler_inputs: &GoSourceCompilerInputs,
        matched: go_practical_capture_binding::GoCompleteRootMatchedExportInputs,
    ) -> Result<go_practical_capture_binding::InstalledGoCompleteRootMatchedCompilerInputs> {
        use go_practical_capture_binding::bind_installed_go_complete_root_matched_inputs;
        use go_practical_installed_compiler::InstalledGoCompiler;
        use std::sync::Arc;

        let semantic = validate_semantic_profile_registry(
            include_bytes!("../../../release/bundles/semantic-profile-registry.json"),
            RegistryRevision::Revision4,
        )?;
        let registry = validate_successor_release_registry(
            include_bytes!("../../../release/bundles/bundle-registry.json"),
            &semantic,
        )?;
        let inventory = &registry
            .registry()
            .toolchain_bundles
            .iter()
            .find(|bundle| bundle.bundle_id == GO_TOOLCHAIN_BUNDLE_ID)
            .ok_or_else(|| fail("registered Go SDK is missing"))?
            .inventory;
        if compiler_inputs.inventory() != inventory {
            return Err(fail("test SDK inventory differs from registered inputs").into());
        }
        let files = compiler_inputs
            .files()
            .iter()
            .map(|file| {
                (
                    file.entry().path.clone(),
                    file.bytes().to_vec(),
                    file.entry().executable,
                )
            })
            .collect();
        let snapshot = frontend_registry::test_snapshot(inventory, files)
            .map_err(|error| fail(format!("test SDK snapshot: {error:?}")))?;
        let compiler = InstalledGoCompiler::test_original_snapshot(&registry, Arc::new(snapshot))
            .map_err(|error| fail(format!("test compiler owner: {error:?}")))?;
        let owner = bind_installed_go_complete_root_matched_inputs(
            compiler,
            compiler_inputs.canonical_bytes(),
            matched,
        )
        .map_err(|error| fail(format!("test installed-root binding: {error:?}")))?;
        if owner.compiler_inputs().canonical_bytes() != compiler_inputs.canonical_bytes()
            || owner.release_registry() != &registry.release_identity()
        {
            return Err(fail("test installed owner differs from original compiler inputs").into());
        }
        Ok(owner)
    }
    fn root_bindings(dir: &Path, export: bool) -> Result<Vec<(String, PathBuf)>> {
        let initial = wait(&dir.join("roots-ready.json"))?;
        if initial["capture_started"] != false
            || export && initial["compiler_output_prepared"] != true
        {
            return Err(fail("root opening was not before the original capture").into());
        }
        let rows = initial["roots"]
            .as_array()
            .ok_or_else(|| fail("missing roots"))?;
        let mut names = rows
            .iter()
            .map(|row| row["Capability"].as_str().unwrap_or(""))
            .collect::<Vec<_>>();
        names.sort_unstable();
        let expected: &[&str] = if export {
            &["original-go-package-exports"]
        } else {
            &["original-go-sdk", "workspace/dep", "workspace/main"]
        };
        if names != expected {
            return Err(fail("the complete original root set differs").into());
        }
        let bindings = rows
            .iter()
            .map(|row| {
                Ok((
                    row["Capability"]
                        .as_str()
                        .ok_or_else(|| fail("missing capability"))?
                        .to_owned(),
                    PathBuf::from(
                        row["Directory"]
                            .as_str()
                            .ok_or_else(|| fail("missing root directory"))?,
                    ),
                ))
            })
            .collect::<std::result::Result<Vec<_>, io::Error>>()?;
        Ok(bindings)
    }
    fn publish_opened(dir: &Path, count: usize) -> Result<()> {
        publish(
            dir,
            "opened.json",
            &json!({"status":"passed","capabilities":count,"opened_before_capture":true}),
        )
    }
    fn observed_snapshot(dir: &Path) -> Result<GoSourceSnapshotInputs> {
        let input = wait(&dir.join("snapshot-ready.json"))?;
        let source_directory = Path::new(
            input["source_directory"]
                .as_str()
                .ok_or_else(|| fail("missing source directory"))?,
        );
        if !source_directory.is_absolute()
            || source_directory
                .file_name()
                .is_none_or(|n| n != "original-source-objects")
        {
            return Err(fail("invalid original source directory").into());
        }
        let observed = snapshot(
            &input,
            source_directory
                .parent()
                .ok_or_else(|| fail("missing evidence directory"))?,
        )?;
        let count = observed.source_objects().len();
        let digest = observed.snapshot_sha256().to_owned();
        if input["original_source_objects"].as_u64() != Some(count as u64)
            || input["native_snapshot_sha256"].as_str() != Some(digest.as_str())
        {
            return Err(fail("original snapshot identity or count differs").into());
        }
        Ok(observed)
    }
    fn publish_matched(dir: &Path, count: usize, digest: &str) -> Result<()> {
        publish(
            dir,
            "matched.json",
            &json!({"status":"passed","opened_before_capture":true,"original_source_objects":count,"native_snapshot_sha256":digest}),
        )
    }
    fn run(source_dir: &Path, export_dir: &Path, combined_dir: &Path) -> Result<()> {
        let source_bindings = root_bindings(source_dir, false)?;
        let source_count = source_bindings.len();
        let roots = GoCaptureSourceRoots::open(source_bindings)
            .map_err(|error| fail(format!("source root open: {error:?}")))?;
        publish_opened(source_dir, source_count)?;
        let export_bindings = root_bindings(export_dir, true)?;
        let export_path = export_bindings
            .into_iter()
            .next()
            .ok_or_else(|| fail("missing export root"))?
            .1;
        let roots = roots
            .open_export(export_path)
            .map_err(|error| fail(format!("export root open: {error:?}")))?;
        publish_opened(export_dir, 1)?;
        let export_snapshot = observed_snapshot(export_dir)?;
        let export_count = export_snapshot.source_objects().len();
        let export_digest = export_snapshot.snapshot_sha256().to_owned();
        let roots = roots
            .match_export(export_snapshot)
            .map_err(|error| fail(format!("export root comparison: {error:?}")))?;
        publish_matched(export_dir, export_count, &export_digest)?;
        let source_snapshot = observed_snapshot(source_dir)?;
        let source_count = source_snapshot.source_objects().len();
        let source_digest = source_snapshot.snapshot_sha256().to_owned();
        let roots = roots
            .match_source(source_snapshot)
            .map_err(|error| fail(format!("source root comparison: {error:?}")))?;
        publish_matched(source_dir, source_count, &source_digest)?;
        let ready = wait(&combined_dir.join("owner-ready.json"))?;
        if ready["original_roots_retired"] != true || ready["export_cache_retired"] != true {
            return Err(fail("original replay retirement was not observed").into());
        }
        let export_start = wait(&export_dir.join("roots-ready.json"))?;
        let export_path = Path::new(
            ready["export_cache_directory"]
                .as_str()
                .ok_or_else(|| fail("missing original export cache path"))?,
        );
        if export_start["roots"][0]["Directory"].as_str() != export_path.to_str() {
            return Err(fail("retired cache differs from the held export root").into());
        }
        match fs::symlink_metadata(export_path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            _ => return Err(fail("the held export cache path still exists").into()),
        }
        let owner_path = Path::new(
            ready["owner_path"]
                .as_str()
                .ok_or_else(|| fail("missing owner path"))?,
        );
        let directory = Path::new(
            ready["source_directory"]
                .as_str()
                .ok_or_else(|| fail("missing owner directory"))?,
        );
        if !owner_path.is_absolute()
            || !directory.is_absolute()
            || owner_path.parent() != Some(directory)
        {
            return Err(fail("invalid original owner path").into());
        }
        let packet: Value = serde_json::from_slice(&fs::read(owner_path)?)?;
        let candidate = candidate_request(&packet, "candidate_request_hex")?;
        let original = exports(&packet, directory)?;
        let matched = roots
            .bind(original)
            .map_err(|error| fail(format!("complete root binding: {error:?}")))?;
        let graph = module_graph(&packet, matched.export_inputs().clone())?;
        {
            let binding = matched
                .bind_candidate_graph(&candidate, &graph)
                .map_err(|error| fail(format!("candidate/source/graph binding: {error:?}")))?;
            let contracts = binding
                .derive_contracts()
                .map_err(|error| fail(format!("candidate source contracts: {error:?}")))?;
            let module = contracts.contracts().module();
            if contracts.contracts().source().source_inputs().roots().len() != 1
                || module.selected_roots().len() != 1
                || module.functions().len() != 1
            {
                return Err(fail("original Go request lost its selected root").into());
            }
            let function = &module.functions()[0];
            if !function.is_selected_root()
                || function.origin() != &GoSourceContractOrigin::Default
                || function.termination() != "partial"
                || !function.requires().is_empty()
                || function.ensures().len() != 1
                || function.ensures()[0].operator() != "bool.literal"
                || function.ensures()[0].literal() != Some(&Value::Bool(true))
            {
                return Err(fail("original zero-overlay default contract changed").into());
            }
            let entries = contracts
                .derive_entry_catalogue()
                .map_err(|error| fail(format!("candidate entry catalogue: {error:?}")))?;
            let module = entries.contracts().contracts().module();
            let layouts = entries.catalogue().layouts();
            if layouts.len() != 1
                || layouts[0].function() != &module.selected_roots()[0]
                || layouts[0].inputs().len() != 1
                || layouts[0].results().len() != 1
            {
                return Err(fail("candidate entry layout lost the original function").into());
            }
            let checked = go_practical_checked_entries::check_go_graph_entries(entries)?;
            let module = checked.entries().contracts().contracts().module();
            let parts = checked.checked().assemblies().parts();
            let reports = checked.checked().reports();
            if parts.len() != 1
                || reports.len() != 1
                || parts[0].function_modules().len() != 1
                || parts[0].function_modules()[0].0 != module.selected_roots()[0]
            {
                return Err(fail("candidate entry assembly lost the original function").into());
            }
            let report: Value =
                serde_json::from_str(&mpk_kernel::render_verification_report_json(&reports[0]))?;
            let mut certificate = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(combined_dir.join("candidate-entry.mpcert"))?;
            certificate.write_all(parts[0].certificate_bytes())?;
            publish(
                combined_dir,
                "candidate-entry-rust.json",
                &json!({"verdict":report["verdict"],"module":report["module"],"declaration_count":report["declaration_count"],"axiom_count":report["axiom_count"],"hashes":report["hashes"]}),
            )?;
            std::hint::black_box(&checked);
        }
        // This optional negative control imports another complete original
        // fixture only in the local test observer; production gets no path or
        // replacement-graph adapter from it.
        #[cfg(test)]
        let crossed_graph_rejected =
            if let Some(directory) = std::env::var_os("MPK_GO_TEST_CROSSED_GRAPH") {
                let directory = PathBuf::from(directory);
                let other_packet: Value = serde_json::from_slice(&fs::read(
                    directory.join("complete-original-module-graph-inputs.json"),
                )?)?;
                let other_graph = module_graph(&other_packet, exports(&other_packet, &directory)?)?;
                let other_export = other_graph
                    .module_data()
                    .module_observations()
                    .export_inputs();
                if other_export
                    .capture_inputs()
                    .snapshot_inputs()
                    .source_objects()
                    .len()
                    != 5667
                {
                    return Err(
                        fail("crossed original graph lost its complete source universe").into(),
                    );
                }
                let other_source = bind_go_practical_source_selection(
                    &candidate,
                    other_export.capture_inputs().selection_inputs(),
                )?;
                if other_source.source_inputs().roots().len() != 1
                    || !matches!(
                        matched.bind_candidate_graph(&candidate, &other_graph),
                        Err(go_practical_capture_binding::GoCandidateRootGraphBindingError::Export)
                    )
                {
                    return Err(fail("crossed original graph was not rejected").into());
                }
                true
            } else {
                false
            };
        #[cfg(not(test))]
        let crossed_graph_rejected = false;
        let source_count = matched.source_snapshot().source_objects().len();
        let export_count = matched.export_snapshot().source_objects().len();
        let total = matched
            .export_inputs()
            .capture_inputs()
            .snapshot_inputs()
            .source_objects()
            .len();
        let compiler = compiler_inputs(&packet, directory, matched.export_inputs().clone())?;
        if compiler.export_inputs().canonical_bytes() != matched.export_inputs().canonical_bytes() {
            return Err(fail("compiler and held roots crossed their original export owner").into());
        }
        let compiler_files = compiler.files().len();
        let compiler_bytes = compiler
            .files()
            .iter()
            .map(|file| file.bytes().len())
            .sum::<usize>();
        if ready["source_objects"].as_u64() != Some(source_count as u64)
            || ready["export_objects"].as_u64() != Some(export_count as u64)
            || ready["total_objects"].as_u64() != Some(total as u64)
            || source_count + export_count != total
        {
            return Err(fail("complete original owner lost source objects").into());
        }
        #[cfg(test)]
        let held_owner = {
            let owner = test_installed_owner(&compiler, matched)?;
            if owner.source_snapshot().source_objects().len() != source_count
                || owner.export_snapshot().source_objects().len() != export_count
            {
                return Err(fail("test installed owner lost original roots").into());
            }
            {
                let binding = owner
                    .bind_candidate_graph(&candidate, &graph)
                    .map_err(|error| {
                        fail(format!(
                            "test installed candidate/source/graph binding: {error:?}"
                        ))
                    })?;
                if binding.source().source_inputs().roots().len() != 1 {
                    return Err(fail("test installed candidate lost its original root").into());
                }
                let other = candidate_request(&packet, "unselected_candidate_request_hex")?;
                if !matches!(
                    owner.bind_candidate_request(&other),
                    Err(GoPracticalSourceBindingError::Mismatch { root: Some(0) })
                ) {
                    return Err(fail(
                        "test installed owner accepted a different original function",
                    )
                    .into());
                }
                std::hint::black_box(&binding);
            }
            owner
        };
        #[cfg(not(test))]
        let held_owner = matched;
        publish(
            combined_dir,
            "owner-matched.json",
            &json!({"status":"passed","both_root_sets_retained":true,"export_cache_retirement_checked_before_binding":true,"source_objects":source_count,"export_objects":export_count,"total_objects":total,"compiler_inputs_imported":true,"registered_sdk_files":compiler_files,"registered_sdk_bytes":compiler_bytes,"candidate_request_bound":true,"candidate_request_sha256":candidate.request_sha256(),"module_graph_verified":true,"candidate_contracts_derived":true,"candidate_entry_math_checked":true,"graph_roots":graph.derived_roots().len(),"graph_expansion_manifests":graph.expansion_manifests().len(),"graph_selected_versions":graph.observed_selected_versions().len(),"test_only_crossed_graph_rejected":crossed_graph_rejected,"test_only_installed_owner_bound":cfg!(test),"test_only_installed_request_bound":cfg!(test),"test_only_installed_other_function_rejected":cfg!(test),"installed_authority_claimed":false,"launcher_authority_claimed":false,"compiler_to_source_proved":false,"application_proof_claimed":false}),
        )?;
        std::hint::black_box((&held_owner, &graph));
        Ok(())
    }

    let arguments = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    assert_eq!(
        arguments.len(),
        3,
        "source, export and owner coordination directories"
    );
    if let Err(error) = run(&arguments[0], &arguments[1], &arguments[2]) {
        for (directory, names) in [
            (&arguments[0], &["opened.json", "matched.json"][..]),
            (&arguments[1], &["opened.json", "matched.json"][..]),
            (&arguments[2], &["owner-matched.json"][..]),
        ] {
            for name in names {
                if !directory.join(name).exists() {
                    let _ = publish(
                        directory,
                        name,
                        &json!({"status":"failed","diagnostic":error.to_string()}),
                    );
                }
            }
        }
        eprintln!("combined-live-root observer: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("the private combined-live-root observer requires Linux");
    std::process::exit(1);
}
