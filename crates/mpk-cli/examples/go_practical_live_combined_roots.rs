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
#[path = "../src/go_practical_checked_components.rs"]
#[allow(dead_code)]
mod go_practical_checked_components;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_entries.rs"]
#[allow(dead_code)]
mod go_practical_checked_entries;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_error_value_relations.rs"]
#[allow(dead_code)]
mod go_practical_checked_error_value_relations;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_invocations.rs"]
#[allow(dead_code)]
mod go_practical_checked_invocations;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_root_strings.rs"]
#[allow(dead_code)]
mod go_practical_checked_root_strings;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_root_values.rs"]
#[allow(dead_code)]
mod go_practical_checked_root_values;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_strings.rs"]
#[allow(dead_code)]
mod go_practical_checked_strings;
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_checked_value_relations.rs"]
#[allow(dead_code)]
mod go_practical_checked_value_relations;
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
    // This constructor is compiled only for the local test observer. It
    // retains the full registered frontend and the independently imported SDK;
    // it does not open or authorize a production installed release.
    #[cfg(test)]
    fn test_registered_release_owner(
        compiler_inputs: &GoSourceCompilerInputs,
        matched: go_practical_capture_binding::GoCompleteRootMatchedExportInputs,
    ) -> Result<go_practical_capture_binding::InstalledGoCompleteRootMatchedReleaseInputs> {
        use go_practical_capture_binding::bind_installed_go_complete_root_matched_release_inputs;
        use go_practical_installed_compiler::InstalledGoRelease;
        use mpk_vc::release_bundle_v1::GO_FRONTEND_BUNDLE_ID;
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
        let frontend = registry
            .registry()
            .frontend_bundles
            .iter()
            .find(|bundle| bundle.bundle_id == GO_FRONTEND_BUNDLE_ID)
            .ok_or_else(|| fail("registered Go frontend is missing"))?;
        if frontend.inventory.files.len() != 1
            || frontend.inventory.files[0].path != frontend.main.path
        {
            return Err(fail("registered Go frontend inventory changed").into());
        }
        let hex = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(
            "../../develop/migrations/go-practical/t02-w01/verification/registered-analysis/rebuild-original/frontend-0.elf.hex",
        ))?;
        let original = unhex(&hex)?;
        let frontend_snapshot = frontend_registry::test_snapshot(
            &frontend.inventory,
            vec![(
                frontend.main.path.clone(),
                original,
                frontend.inventory.files[0].executable,
            )],
        )
        .map_err(|error| fail(format!("test frontend snapshot: {error:?}")))?;
        let release = InstalledGoRelease::test_original_snapshots(
            &registry,
            Arc::new(frontend_snapshot),
            Arc::new(snapshot),
        )
        .map_err(|error| fail(format!("test release owner: {error:?}")))?;
        let owner = bind_installed_go_complete_root_matched_release_inputs(
            release,
            compiler_inputs.canonical_bytes(),
            matched,
        )
        .map_err(|error| fail(format!("test release-root binding: {error:?}")))?;
        if owner.compiler().compiler_inputs().canonical_bytes() != compiler_inputs.canonical_bytes()
            || owner.compiler().release_registry() != &registry.release_identity()
            || owner.frontend().bundle_id() != GO_FRONTEND_BUNDLE_ID
            || owner.frontend().inventory() != &frontend.inventory
            || owner.frontend().main_path() != frontend.main.path
            || owner
                .frontend()
                .snapshot()
                .file(owner.frontend().main_path())
                .map(|file| file.bytes.len())
                != Some(9_004_378)
        {
            return Err(fail("test release owner differs from registered original inputs").into());
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
        #[cfg(test)]
        let expect_call = match std::env::var("MPK_GO_TEST_EXPECT_CALLS") {
            Ok(value) if value == "1" => true,
            Err(std::env::VarError::NotPresent) => false,
            _ => return Err(fail("invalid test call expectation").into()),
        };
        #[cfg(not(test))]
        let expect_call = false;
        #[cfg(test)]
        let expect_overlay = match std::env::var("MPK_GO_TEST_EXPECT_OVERLAY") {
            Ok(value) if value == "1" => true,
            Err(std::env::VarError::NotPresent) => false,
            _ => return Err(fail("invalid test overlay expectation").into()),
        };
        #[cfg(not(test))]
        let expect_overlay = false;
        #[cfg(test)]
        let expect_error = match std::env::var("MPK_GO_TEST_EXPECT_ERROR") {
            Ok(value) if value == "1" => true,
            Err(std::env::VarError::NotPresent) => false,
            _ => return Err(fail("invalid test error expectation").into()),
        };
        #[cfg(not(test))]
        let expect_error = false;
        if expect_overlay && (!expect_call || expect_error) {
            return Err(fail("invalid source-overlay test combination").into());
        }
        let expected_functions = if expect_call { 2 } else { 1 };
        let expected_calls = usize::from(expect_call);
        let original = exports(&packet, directory)?;
        let matched = roots
            .bind(original)
            .map_err(|error| fail(format!("complete root binding: {error:?}")))?;
        let graph = module_graph(&packet, matched.export_inputs().clone())?;
        let string_receipts = {
            let root_strings = go_practical_checked_root_strings::check_go_root_string_components(
                &matched, &candidate, &graph,
            )?;
            let root_values = root_strings.values();
            let components = root_values.components();
            let invocations = components.invocations();
            if invocations.invocations().catalogue().sites().len() != expected_calls
                || invocations.checked().receipts().len() != expected_calls
            {
                return Err(fail("original call closure changed").into());
            }
            let checked = components.entries();
            let contracts = checked.entries().contracts();
            let module = contracts.contracts().module();
            let strings = root_strings.strings();
            if strings.binding().contracts().contracts().module() != module {
                return Err(fail("source strings crossed original contracts").into());
            }
            let receipts = strings.checked().receipts();
            if (!expect_error && !receipts.is_empty())
                || (expect_error
                    && expect_call
                    && (receipts.len() != 1
                        || !module.value_domain().types().any(|key| {
                            matches!(key, GoSourceTypeKey::Builtin { name, .. }
                                if name == "string" && key.id() == receipts[0].source_type_id())
                        })))
            {
                return Err(fail("original source-string closure changed").into());
            }
            for (index, receipt) in receipts.iter().enumerate() {
                let row = strings.reconstruct(index)?;
                let mut certificate = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(combined_dir.join(format!("source-string-{index}.mpcert")))?;
                certificate.write_all(row.certificate_bytes())?;
                publish(
                    combined_dir,
                    &format!("source-string-{index}-rust.json"),
                    &json!({"verdict":"accepted","module":receipt.module(),"declaration_count":receipt.declarations(),"axiom_count":0,"hashes":{"export":mpk_cert::hash_hex(receipt.export_hash()),"axiom_report":mpk_cert::hash_hex(receipt.axiom_report_hash()),"certificate":mpk_cert::hash_hex(receipt.certificate_hash())}}),
                )?;
            }
            let string_receipts = receipts
                .iter()
                .map(|receipt| {
                    (
                        receipt.source_type_id().to_string(),
                        *receipt.certificate_hash(),
                    )
                })
                .collect::<Vec<_>>();
            if contracts.contracts().source().source_inputs().roots().len() != 1
                || module.selected_roots().len() != 1
                || module.functions().len() != expected_functions
            {
                return Err(fail("original Go request lost its selected root").into());
            }
            if module
                .functions()
                .iter()
                .filter(|f| f.is_selected_root())
                .count()
                != 1
                || module.functions().iter().any(|function| {
                    function.termination() != "partial"
                        || !function.requires().is_empty()
                        || function.ensures().len() != 1
                        || if (expect_overlay || expect_error) && function.is_selected_root() {
                            !matches!(
                                function.origin(),
                                GoSourceContractOrigin::SourceOverlay { .. }
                            ) || if expect_error {
                                function.ensures()[0].operator() != "bool.literal"
                                    || function.ensures()[0].literal() != Some(&Value::Bool(true))
                            } else {
                                function.ensures()[0].operator() != "eq"
                                    || function.ensures()[0].children().len() != 2
                                    || function.ensures()[0].children()[0].operator() != "result"
                                    || function.ensures()[0].children()[1].operator() != "input"
                            }
                        } else {
                            function.origin() != &GoSourceContractOrigin::Default
                                || function.ensures()[0].operator() != "bool.literal"
                                || function.ensures()[0].literal() != Some(&Value::Bool(true))
                        }
                })
            {
                return Err(fail("original source contracts changed").into());
            }
            if expect_overlay || expect_error {
                let captured = module.captured_overlays();
                if captured.len() != 1
                    || !captured[0].in_body_closure()
                    || captured[0].function() != &module.selected_roots()[0]
                    || captured[0].overlay().source_path() != "policy.go"
                {
                    return Err(fail("original selected overlay was lost").into());
                }
                let raw = captured[0].overlay().source_buffer();
                let expected_overlay = if expect_error {
                    let function = if expect_call {
                        "RootErrorText"
                    } else {
                        "RootWithError"
                    };
                    json!({"schema":"mpk.go.practical.source_overlay.v1","source":"policy.go","function":{"package":"example.com/policy","function":function,"receiver":null},"termination":"partial","requires":[],"ensures":[{"op":"bool.literal","value":true}],"error_inputs":[{"input":0,"path":[],"cases":["example.com/policy.Refusal"]}],"loops":[],"modifies":[]})
                } else {
                    json!({"schema":"mpk.go.practical.source_overlay.v1","source":"policy.go","function":{"package":"example.com/policy","function":"RootWithCall","receiver":null},"termination":"partial","requires":[],"ensures":[{"op":"eq","args":[{"op":"result","position":0},{"op":"input","position":0}]}],"error_inputs":[],"loops":[],"modifies":[]})
                };
                if raw.object().capability != "workspace/main"
                    || raw.object().path != "contracts/root.json"
                    || serde_json::from_slice::<Value>(raw.bytes())? != expected_overlay
                    || module
                        .functions()
                        .iter()
                        .find(|function| function.is_selected_root())
                        .and_then(|function| function.overlay_object_id())
                        != Some(raw.object().id.as_str())
                {
                    return Err(fail("original selected overlay identity changed").into());
                }
            } else if !module.captured_overlays().is_empty() {
                return Err(fail("unexpected original overlay").into());
            }
            if expect_error {
                let original = module
                    .functions()
                    .iter()
                    .find(|function| function.is_selected_root())
                    .ok_or_else(|| fail("missing original error entry"))?;
                let domain = module.value_domain();
                if domain.error_cases().len() != 1
                    || original.error_inputs().len() != 1
                    || original.error_inputs()[0].input_position() != 0
                    || !original.error_inputs()[0].path().is_empty()
                    || original.error_inputs()[0].case_type_ids()
                        != [domain.error_cases()[0].value_type_id()]
                {
                    return Err(fail("original closed error contract changed").into());
                }
                let case_id = domain.error_cases()[0].value_type_id().to_string();
                let checked = match root_values.values() {
                    go_practical_checked_root_values::GoCheckedRootValues::Error(value) => value,
                    _ => return Err(fail("original error domain used the wrong value path").into()),
                };
                if checked.binding().contracts().contracts().module() != module {
                    return Err(fail("error relations crossed original contracts").into());
                }
                let receipts = checked.checked().receipts();
                if receipts.len() != 2
                    || receipts[0].boxed_source_type_id().is_some()
                    || receipts[1].boxed_source_type_id() != Some(case_id.as_str())
                {
                    return Err(fail("original error value rows were not all checked").into());
                }
                for (index, label) in ["nil", "case"].into_iter().enumerate() {
                    let row = checked.reconstruct(index)?;
                    let receipt = &receipts[index];
                    let mut certificate = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(combined_dir.join(format!("error-value-{label}.mpcert")))?;
                    certificate.write_all(row.certificate_bytes())?;
                    publish(
                        combined_dir,
                        &format!("error-value-{label}-rust.json"),
                        &json!({"verdict":"accepted","module":receipt.module(),"declaration_count":receipt.declarations(),"axiom_count":0,"hashes":{"export":mpk_cert::hash_hex(receipt.export_hash()),"axiom_report":mpk_cert::hash_hex(receipt.axiom_report_hash()),"certificate":mpk_cert::hash_hex(receipt.certificate_hash())}}),
                    )?;
                }
            } else {
                let checked = match root_values.values() {
                    go_practical_checked_root_values::GoCheckedRootValues::ErrorFree(value) => {
                        value
                    }
                    _ => {
                        return Err(
                            fail("original error-free domain used the wrong value path").into()
                        )
                    }
                };
                if checked.binding().contracts().contracts().module() != module
                    || checked.report().axiom_count != 0
                {
                    return Err(fail("error-free relations crossed original contracts").into());
                }
                let mut certificate = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(combined_dir.join("root-value-relations.mpcert"))?;
                certificate.write_all(checked.binding().relations().certificate_bytes())?;
                let report: Value = serde_json::from_str(
                    &mpk_kernel::render_verification_report_json(checked.report()),
                )?;
                publish(
                    combined_dir,
                    "root-value-relations-rust.json",
                    &json!({"verdict":report["verdict"],"module":report["module"],"declaration_count":report["declaration_count"],"axiom_count":report["axiom_count"],"hashes":report["hashes"]}),
                )?;
            }
            if expect_call {
                let site = &invocations.invocations().catalogue().sites()[0];
                if site.caller() != &module.selected_roots()[0]
                    || site.targets().len() != 1
                    || site.input_type_ids().len() != 1
                    || site.result_type_ids().len() != 1
                    || !module.functions().iter().any(|function| {
                        !function.is_selected_root()
                            && function.function() == site.targets()[0].callee()
                    })
                {
                    return Err(fail("original cross-package call changed").into());
                }
                let callee = module
                    .selection()
                    .source(site.targets()[0].callee().source_object_id())
                    .ok_or_else(|| fail("missing original call target source"))?;
                if expect_error {
                    if !site.is_error_dispatch()
                        || site.targets()[0].dynamic_type_id()
                            != Some(module.value_domain().error_cases()[0].value_type_id())
                        || callee.source().object().capability != "workspace/main"
                        || callee.source().object().path != "policy.go"
                    {
                        return Err(fail("closed error dispatch target changed").into());
                    }
                } else if site.is_error_dispatch()
                    || callee.source().object().capability != "workspace/dep"
                    || callee.source().object().path != "example.com/c@v1.1.0/dep.go"
                {
                    return Err(fail("cross-package call target differs from original c").into());
                }
                let row = invocations.reconstruct(0)?;
                let receipt = &invocations.checked().receipts()[0];
                let mut certificate = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(combined_dir.join("candidate-invocation.mpcert"))?;
                certificate.write_all(row.certificate_bytes())?;
                publish(
                    combined_dir,
                    "candidate-invocation-rust.json",
                    &json!({"verdict":"accepted","module":receipt.module(),"declaration_count":receipt.declarations(),"axiom_count":0,"hashes":{"export":mpk_cert::hash_hex(receipt.export_hash()),"axiom_report":mpk_cert::hash_hex(receipt.axiom_report_hash()),"certificate":mpk_cert::hash_hex(receipt.certificate_hash())}}),
                )?;
            }
            let layouts = checked.entries().catalogue().layouts();
            if layouts.len() != expected_functions
                || !layouts
                    .iter()
                    .any(|layout| layout.function() == &module.selected_roots()[0])
                || layouts
                    .iter()
                    .any(|layout| layout.inputs().len() != 1 || layout.results().len() != 1)
            {
                return Err(fail("candidate entry layout lost the original function").into());
            }
            let parts = checked.checked().assemblies().parts();
            let reports = checked.checked().reports();
            if parts.len() != 1
                || reports.len() != 1
                || parts[0].function_modules().len() != expected_functions
                || !parts[0]
                    .function_modules()
                    .iter()
                    .any(|(function, _)| function == &module.selected_roots()[0])
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
            std::hint::black_box(&components);
            string_receipts
        };
        // This optional negative control imports another complete original
        // fixture only in the local test observer; production gets no path or
        // replacement-graph adapter from it.
        #[cfg(test)]
        let crossed_graph_rejected = if let Some(directory) =
            std::env::var_os("MPK_GO_TEST_CROSSED_GRAPH")
        {
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
            let baseline = candidate_request(&packet, "baseline_candidate_request_hex")?;
            let other_source = bind_go_practical_source_selection(
                &baseline,
                other_export.capture_inputs().selection_inputs(),
            )?;
            if other_source.source_inputs().roots().len() != 1
                    || !matches!(
                        matched.bind_candidate_graph(&candidate, &other_graph),
                        Err(go_practical_capture_binding::GoCandidateRootGraphBindingError::Export)
                    )
                    || !matches!(
                        go_practical_checked_components::check_go_root_components(
                            &matched,
                            &candidate,
                            &other_graph
                        ),
                        Err(go_practical_checked_components::GoCheckedComponentsError::Binding(
                            go_practical_capture_binding::GoCandidateRootGraphBindingError::Export
                        ))
                    )
                    || !matches!(
                        go_practical_checked_root_values::check_go_root_value_components(
                            &matched,
                            &candidate,
                            &other_graph
                        ),
                        Err(go_practical_checked_root_values::GoCheckedRootValueError::Components(
                            go_practical_checked_components::GoCheckedComponentsError::Binding(
                                go_practical_capture_binding::GoCandidateRootGraphBindingError::Export
                            )
                        ))
                    )
                    || !matches!(
                        go_practical_checked_root_strings::check_go_root_string_components(
                            &matched, &candidate, &other_graph
                        ),
                        Err(go_practical_checked_root_strings::GoCheckedRootStringError::Values(
                            go_practical_checked_root_values::GoCheckedRootValueError::Components(
                                go_practical_checked_components::GoCheckedComponentsError::Binding(
                                    go_practical_capture_binding::GoCandidateRootGraphBindingError::Export
                                )
                            )
                        ))
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
            let release = test_registered_release_owner(&compiler, matched)?;
            let owner = release.compiler();
            if owner.source_snapshot().source_objects().len() != source_count
                || owner.export_snapshot().source_objects().len() != export_count
            {
                return Err(fail("test installed owner lost original roots").into());
            }
            {
                let installed_strings =
                    go_practical_checked_root_strings::check_installed_go_root_string_components(
                        owner, &candidate, &graph,
                    )?;
                let installed_values = installed_strings.values();
                let installed_receipts = installed_strings
                    .strings()
                    .checked()
                    .receipts()
                    .iter()
                    .map(|receipt| {
                        (
                            receipt.source_type_id().to_string(),
                            *receipt.certificate_hash(),
                        )
                    })
                    .collect::<Vec<_>>();
                if installed_receipts != string_receipts {
                    return Err(
                        fail("test installed owner changed source-string certificates").into(),
                    );
                }
                let components = installed_values.components();
                if components.entries().checked().assemblies().parts().len() != 1
                    || components.entries().checked().assemblies().parts()[0]
                        .function_modules()
                        .len()
                        != expected_functions
                    || components.invocations().checked().receipts().len() != expected_calls
                {
                    return Err(fail("test installed components lost the original root").into());
                }
                match (expect_error, installed_values.values()) {
                    (
                        true,
                        go_practical_checked_root_values::GoCheckedRootValues::Error(values),
                    ) if values.checked().receipts().len() == 2 => {}
                    (
                        false,
                        go_practical_checked_root_values::GoCheckedRootValues::ErrorFree(values),
                    ) if values.report().axiom_count == 0 => {}
                    _ => return Err(fail("test installed owner lost the value relations").into()),
                }
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
                ) || !matches!(
                    go_practical_checked_components::check_installed_go_root_components(
                        owner, &other, &graph
                    ),
                    Err(
                        go_practical_checked_components::GoCheckedComponentsError::Binding(
                            go_practical_capture_binding::GoCandidateRootGraphBindingError::Source(
                                GoPracticalSourceBindingError::Mismatch { root: Some(0) }
                            )
                        )
                    )
                ) || !matches!(
                    go_practical_checked_root_values::check_installed_go_root_value_components(
                        owner, &other, &graph
                    ),
                    Err(
                        go_practical_checked_root_values::GoCheckedRootValueError::Components(
                            go_practical_checked_components::GoCheckedComponentsError::Binding(
                                go_practical_capture_binding::GoCandidateRootGraphBindingError::Source(
                                    GoPracticalSourceBindingError::Mismatch { root: Some(0) }
                                )
                            )
                        )
                    )
                ) || !matches!(
                    go_practical_checked_root_strings::check_installed_go_root_string_components(
                        owner, &other, &graph
                    ),
                    Err(go_practical_checked_root_strings::GoCheckedRootStringError::Values(
                        go_practical_checked_root_values::GoCheckedRootValueError::Components(
                            go_practical_checked_components::GoCheckedComponentsError::Binding(
                                go_practical_capture_binding::GoCandidateRootGraphBindingError::Source(
                                    GoPracticalSourceBindingError::Mismatch { root: Some(0) }
                                )
                            )
                        )
                    ))
                ) {
                    return Err(fail(
                        "test installed owner accepted a different original function",
                    )
                    .into());
                }
                std::hint::black_box((&binding, &components));
            }
            release
        };
        #[cfg(not(test))]
        let held_owner = matched;
        publish(
            combined_dir,
            "owner-matched.json",
            &json!({"status":"passed","both_root_sets_retained":true,"export_cache_retirement_checked_before_binding":true,"source_objects":source_count,"export_objects":export_count,"total_objects":total,"compiler_inputs_imported":true,"registered_sdk_files":compiler_files,"registered_sdk_bytes":compiler_bytes,"candidate_request_bound":true,"candidate_request_sha256":candidate.request_sha256(),"module_graph_verified":true,"candidate_contracts_derived":true,"candidate_entry_math_checked":true,"candidate_invocations_checked":true,"candidate_function_bodies":expected_functions,"candidate_call_sites":expected_calls,"test_overlay_bound":expect_overlay,"test_error_contract_bound":expect_error,"same_root_components_checked":true,"same_root_value_components_checked":true,"same_root_string_components_checked":true,"source_string_rows":string_receipts.len(),"graph_roots":graph.derived_roots().len(),"graph_expansion_manifests":graph.expansion_manifests().len(),"graph_selected_versions":graph.observed_selected_versions().len(),"test_only_crossed_graph_rejected":crossed_graph_rejected,"test_only_installed_owner_bound":cfg!(test),"test_only_installed_frontend_retained":cfg!(test),"test_only_installed_components_checked":cfg!(test),"test_only_installed_values_checked":cfg!(test),"test_only_installed_strings_checked":cfg!(test),"test_only_installed_request_bound":cfg!(test),"test_only_installed_other_function_rejected":cfg!(test),"installed_authority_claimed":false,"launcher_authority_claimed":false,"compiler_to_source_proved":false,"application_proof_claimed":false}),
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
