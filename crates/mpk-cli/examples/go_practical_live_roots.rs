//! Local test observer for original Go capture roots; this is not a CLI route.
#[cfg(target_os = "linux")]
#[path = "../src/go_practical_source_roots.rs"]
mod go_practical_source_roots;

#[cfg(target_os = "linux")]
fn main() {
    use go_practical_source_roots::GoSourceRoots;
    use mpk_vc::go_practical_type_keys::{import_go_source_snapshot_inputs, GoTypeSourceBuffer};
    use serde_json::{json, Value};
    use std::fs::{self, OpenOptions};
    use std::io::{self, Write};
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, Instant};

    fn fail(message: impl ToString) -> io::Error {
        io::Error::other(message.to_string())
    }
    fn read_json(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }
    fn publish(dir: &Path, name: &str, data: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let temporary = dir.join(format!(".rust-root-observer-{}-{name}", std::process::id()));
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
    fn wait(path: &Path) -> Result<Value, Box<dyn std::error::Error>> {
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
        Err(fail("original Go snapshot did not arrive").into())
    }
    fn unhex(text: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let pairs = text.as_bytes().chunks_exact(2);
        if !pairs.remainder().is_empty() {
            return Err(fail("odd hexadecimal input").into());
        }
        pairs
            .map(|pair| Ok(u8::from_str_radix(std::str::from_utf8(pair)?, 16)?))
            .collect()
    }
    fn run(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let initial = read_json(&dir.join("roots-ready.json"))?;
        if initial["capture_started"] != false {
            return Err(fail("root handles would open after capture began").into());
        }
        let rows = initial["roots"]
            .as_array()
            .ok_or_else(|| fail("missing roots"))?;
        if rows.len() != 3 {
            return Err(fail("complete application, dependency and SDK roots are required").into());
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
            .collect::<Result<Vec<_>, io::Error>>()?;
        let roots =
            GoSourceRoots::open(bindings).map_err(|error| fail(format!("root open: {error:?}")))?;
        publish(
            dir,
            "opened.json",
            &json!({"status":"passed","capabilities":rows.len()}),
        )?;
        let input = wait(&dir.join("snapshot-ready.json"))?;
        let source_directory = Path::new(
            input["source_directory"]
                .as_str()
                .ok_or_else(|| fail("missing original source directory"))?,
        );
        if !source_directory.is_absolute() {
            return Err(fail("original source directory is not absolute").into());
        }
        let rows = input["snapshot_sources"]
            .as_array()
            .ok_or_else(|| fail("missing original source objects"))?;
        let mut sources = Vec::with_capacity(rows.len());
        for row in rows {
            let filename = row["raw_file"]
                .as_str()
                .ok_or_else(|| fail("missing raw source file"))?;
            let name_bytes = filename.as_bytes();
            if name_bytes.len() != 68
                || !name_bytes.ends_with(b".raw")
                || !name_bytes[..64].iter().all(u8::is_ascii_hexdigit)
            {
                return Err(fail("invalid original source filename").into());
            }
            let raw = fs::read(source_directory.join(filename))?;
            sources.push(GoTypeSourceBuffer::import(
                &serde_json::to_vec(&row["object"])?,
                &raw,
            )?);
        }
        if sources.len() < 5480
            || input["original_source_objects"].as_u64() != Some(sources.len() as u64)
        {
            return Err(fail("original application or SDK source set narrowed").into());
        }
        let snapshot = import_go_source_snapshot_inputs(
            &unhex(
                input["snapshot_hex"]
                    .as_str()
                    .ok_or_else(|| fail("missing native snapshot"))?,
            )?,
            &sources,
        )?;
        let dirs = snapshot.observed_directories().count();
        let entries = snapshot
            .observed_directories()
            .map(|(_, _, entries)| entries.len())
            .sum::<usize>();
        let original_digest = snapshot.snapshot_sha256().to_owned();
        if input["native_snapshot_sha256"].as_str() != Some(original_digest.as_str()) {
            return Err(fail("native snapshot identity differs").into());
        }
        let matched = roots
            .compare(snapshot)
            .map_err(|error| fail(format!("root comparison: {error:?}")))?;
        if matched.snapshot().source_objects().len() != sources.len() {
            return Err(fail("root comparison lost original sources").into());
        }
        publish(
            dir,
            "matched.json",
            &json!({"status":"passed","opened_before_capture":true,"original_source_objects":sources.len(),"observed_directories":dirs,"observed_entries":entries,"native_snapshot_sha256":original_digest,"authorized_launcher_claimed":false,"application_proof_claimed":false}),
        )?;
        Ok(())
    }
    let dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("coordination directory");
    if let Err(error) = run(&dir) {
        let name = if dir.join("opened.json").exists() {
            "matched.json"
        } else {
            "opened.json"
        };
        let _ = publish(
            &dir,
            name,
            &json!({"status":"failed","diagnostic":error.to_string()}),
        );
        eprintln!("live-root observer: {error}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("the private live-root observer requires Linux");
    std::process::exit(1);
}
