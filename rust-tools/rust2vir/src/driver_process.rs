use crate::driver_protocol::{
    DriverProtocolCode, DriverProtocolError, DriverRequest, OUTPUT_TRANSPORT_MAX,
    REQUEST_TRANSPORT_MAX,
};
use crate::sha256::{digest, hex};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

#[cfg(unix)]
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

const PARTIAL_FILENAME: &str = "result.json.partial";
const FINAL_FILENAME: &str = "result.json";
const FIXED_BINARY_BYTES_MAX: usize = 268_435_456;

#[cfg(unix)]
type RegularIdentity = (u64, u64, u64, i64, i64, i64, i64);
#[cfg(not(unix))]
type RegularIdentity = (u64, u64, u64);

#[derive(Clone, Copy)]
enum RegularMode {
    Request,
    Binary,
    Output,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WrapperInvocation {
    VersionProbe,
    SysrootProbe,
    CrateInformationHost,
    CrateInformationTarget,
    Primary,
    PrimaryArgumentMismatch,
}

pub fn classify_invocation(
    request: &DriverRequest,
    arguments: &[String],
) -> Result<WrapperInvocation, DriverProtocolError> {
    if arguments.first().map(String::as_str) != Some("/mpk/toolchain/bin/rustc")
        || arguments.iter().any(|argument| {
            argument.is_empty() || argument.contains('\0') || argument.starts_with('@')
        })
    {
        return Err(DriverProtocolCode::Identity.into());
    }
    if arguments == ["/mpk/toolchain/bin/rustc", "-vV"] {
        return Ok(WrapperInvocation::VersionProbe);
    }
    if arguments == ["/mpk/toolchain/bin/rustc", "--print", "sysroot"] {
        return Ok(WrapperInvocation::SysrootProbe);
    }
    if arguments == crate_information_host() {
        return Ok(WrapperInvocation::CrateInformationHost);
    }
    let target_probe = crate_information_target(request.target());
    if arguments == target_probe {
        return Ok(WrapperInvocation::CrateInformationTarget);
    }
    if primary_matches(request, arguments) {
        return Ok(WrapperInvocation::Primary);
    }
    if primary_with_extra_arguments(request, arguments) {
        return Ok(WrapperInvocation::PrimaryArgumentMismatch);
    }
    Err(DriverProtocolCode::Identity.into())
}

pub fn read_request(path: &Path) -> Result<Vec<u8>, DriverProtocolError> {
    read_stable_regular(path, REQUEST_TRANSPORT_MAX, RegularMode::Request)
}

pub fn validate_fixed_binary_identities(
    request: &DriverRequest,
) -> Result<(), DriverProtocolError> {
    for (path, expected) in [
        (
            Path::new("/mpk/frontend/rust2vir-driver"),
            request.driver_binary_sha256(),
        ),
        (
            Path::new("/mpk/toolchain/bin/rustc"),
            request.compiler_binary_sha256(),
        ),
    ] {
        let bytes = read_stable_regular(path, FIXED_BINARY_BYTES_MAX, RegularMode::Binary)?;
        if hex(&digest(&bytes)) != expected {
            return Err(DriverProtocolCode::Identity.into());
        }
    }
    Ok(())
}

pub fn consume_result(directory: &Path) -> Result<Vec<u8>, DriverProtocolError> {
    let directory = open_directory(directory)?;
    consume_result_from_open_directory(&directory)
}

pub fn consume_result_from_open_directory(
    directory: &File,
) -> Result<Vec<u8>, DriverProtocolError> {
    validate_open_directory(directory)?;
    let root = directory_handle_path(directory);
    let names = directory_names(&root)?;
    if names != [FINAL_FILENAME] {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    read_stable_regular(
        &root.join(FINAL_FILENAME),
        OUTPUT_TRANSPORT_MAX,
        RegularMode::Output,
    )
}

pub fn publish_result(directory: &Path, bytes: &[u8]) -> Result<(), DriverProtocolError> {
    publish_result_inner(directory, bytes, DriverProtocolCode::Filesystem)
}

pub fn publish_primary_result(directory: &Path, bytes: &[u8]) -> Result<(), DriverProtocolError> {
    publish_result_inner(directory, bytes, DriverProtocolCode::Count)
}

fn publish_result_inner(
    directory: &Path,
    bytes: &[u8],
    primary_collision: DriverProtocolCode,
) -> Result<(), DriverProtocolError> {
    if bytes.len() > OUTPUT_TRANSPORT_MAX {
        return Err(DriverProtocolCode::OutputLimit.into());
    }
    let directory = open_directory(directory)?;
    let root = directory_handle_path(&directory);
    let initial_names = directory_names(&root)?;
    if !initial_names.is_empty() {
        let code = if primary_collision == DriverProtocolCode::Count
            && matches!(initial_names.as_slice(), [name] if matches!(name.as_str(), PARTIAL_FILENAME | FINAL_FILENAME))
        {
            DriverProtocolCode::Count
        } else {
            DriverProtocolCode::Filesystem
        };
        return Err(code.into());
    }
    let partial_path = root.join(PARTIAL_FILENAME);
    let final_path = root.join(FINAL_FILENAME);
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600).custom_flags(platform::O_NOFOLLOW);
    let mut partial = options.open(&partial_path).map_err(|error| {
        if primary_collision == DriverProtocolCode::Count
            && error.kind() == std::io::ErrorKind::AlreadyExists
        {
            DriverProtocolCode::Count
        } else {
            DriverProtocolCode::Filesystem
        }
    })?;
    let before = file_node_identity(&partial)?;
    partial
        .write_all(bytes)
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    partial
        .flush()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    partial
        .sync_all()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    if file_node_identity(&partial)? != before
        || partial
            .metadata()
            .map_err(|_| DriverProtocolCode::Filesystem)?
            .len()
            != bytes.len() as u64
        || fs::symlink_metadata(&final_path).is_ok()
    {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    platform::rename_no_replace(&partial_path, &final_path)
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    directory
        .sync_all()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    drop(partial);
    let published = consume_result_from_open_directory(&directory)?;
    if published != bytes {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok(())
}

pub fn open_directory_is_empty(directory: &File) -> Result<bool, DriverProtocolError> {
    validate_open_directory(directory)?;
    Ok(directory_names(&directory_handle_path(directory))?.is_empty())
}

fn directory_names(directory: &Path) -> Result<Vec<String>, DriverProtocolError> {
    let mut names = fs::read_dir(directory)
        .map_err(|_| DriverProtocolCode::Filesystem)?
        .map(|entry| {
            entry
                .map_err(|_| DriverProtocolCode::Filesystem)
                .and_then(|entry| {
                    entry
                        .file_name()
                        .into_string()
                        .map_err(|_| DriverProtocolCode::Filesystem)
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    names.sort();
    Ok(names)
}

fn read_stable_regular(
    path: &Path,
    maximum: usize,
    mode: RegularMode,
) -> Result<Vec<u8>, DriverProtocolError> {
    let named_before = fs::symlink_metadata(path).map_err(|_| DriverProtocolCode::Filesystem)?;
    if !named_before.is_file() || named_before.file_type().is_symlink() {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    #[cfg(unix)]
    if named_before.nlink() != 1 || !valid_regular_mode(&named_before, mode) {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(platform::O_NOFOLLOW);
    let mut file = options
        .open(path)
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    let opened_before = file_identity(&file)?;
    if opened_before != metadata_identity(&named_before)? {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    let maximum_u64 = u64::try_from(maximum).map_err(|_| limit_code(mode))?;
    if named_before.len() > maximum_u64 {
        return Err(limit_code(mode).into());
    }
    let capacity = usize::try_from(named_before.len()).map_err(|_| limit_code(mode))?;
    let mut bytes = Vec::with_capacity(capacity);
    Read::by_ref(&mut file)
        .take(maximum_u64.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    if bytes.len() > maximum {
        return Err(limit_code(mode).into());
    }
    file.seek(SeekFrom::Start(0))
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    let opened_after = file_identity(&file)?;
    let named_after = fs::symlink_metadata(path).map_err(|_| DriverProtocolCode::Filesystem)?;
    if opened_before != opened_after
        || opened_after != metadata_identity(&named_after)?
        || named_after.len() != bytes.len() as u64
    {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok(bytes)
}

fn open_directory(directory: &Path) -> Result<File, DriverProtocolError> {
    let named = fs::symlink_metadata(directory).map_err(|_| DriverProtocolCode::Filesystem)?;
    if !named.is_dir() || named.file_type().is_symlink() {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    options.custom_flags(platform::O_NOFOLLOW | platform::O_DIRECTORY);
    let opened = options
        .open(directory)
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    validate_open_directory(&opened)?;
    #[cfg(unix)]
    if directory_identity(&named)?
        != directory_identity(
            &opened
                .metadata()
                .map_err(|_| DriverProtocolCode::Filesystem)?,
        )?
    {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok(opened)
}

fn validate_open_directory(directory: &File) -> Result<(), DriverProtocolError> {
    let metadata = directory
        .metadata()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    if !metadata.is_dir() {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    #[cfg(unix)]
    if metadata.nlink() < 2 || metadata.permissions().mode() & 0o7777 != 0o700 {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok(())
}

#[cfg(unix)]
fn directory_handle_path(directory: &File) -> std::path::PathBuf {
    #[cfg(target_os = "linux")]
    let prefix = "/proc/self/fd";
    #[cfg(not(target_os = "linux"))]
    let prefix = "/dev/fd";
    Path::new(prefix).join(directory.as_raw_fd().to_string())
}

fn limit_code(mode: RegularMode) -> DriverProtocolCode {
    if matches!(mode, RegularMode::Output) {
        DriverProtocolCode::OutputLimit
    } else {
        DriverProtocolCode::Transport
    }
}

#[cfg(unix)]
fn valid_regular_mode(metadata: &fs::Metadata, mode: RegularMode) -> bool {
    let permissions = metadata.permissions().mode() & 0o7777;
    match mode {
        RegularMode::Request => permissions == 0o400,
        RegularMode::Binary => permissions & 0o222 == 0 && permissions & 0o111 != 0,
        RegularMode::Output => permissions == 0o600,
    }
}

#[cfg(not(unix))]
fn valid_regular_mode(_metadata: &fs::Metadata, _mode: RegularMode) -> bool {
    true
}

#[cfg(unix)]
fn directory_identity(metadata: &fs::Metadata) -> Result<(u64, u64), DriverProtocolError> {
    if !metadata.is_dir() {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn file_identity(file: &File) -> Result<RegularIdentity, DriverProtocolError> {
    let metadata = file
        .metadata()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((
        metadata.dev(),
        metadata.ino(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    ))
}

#[cfg(unix)]
fn file_node_identity(file: &File) -> Result<(u64, u64), DriverProtocolError> {
    let metadata = file
        .metadata()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((metadata.dev(), metadata.ino()))
}

#[cfg(unix)]
fn metadata_identity(metadata: &fs::Metadata) -> Result<RegularIdentity, DriverProtocolError> {
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((
        metadata.dev(),
        metadata.ino(),
        metadata.len(),
        metadata.mtime(),
        metadata.mtime_nsec(),
        metadata.ctime(),
        metadata.ctime_nsec(),
    ))
}

#[cfg(not(unix))]
fn file_identity(file: &File) -> Result<RegularIdentity, DriverProtocolError> {
    let metadata = file
        .metadata()
        .map_err(|_| DriverProtocolCode::Filesystem)?;
    if !metadata.is_file() {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((0, 0, metadata.len()))
}

#[cfg(not(unix))]
fn file_node_identity(file: &File) -> Result<(u64, u64), DriverProtocolError> {
    if !file
        .metadata()
        .map_err(|_| DriverProtocolCode::Filesystem)?
        .is_file()
    {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((0, 0))
}

#[cfg(not(unix))]
fn metadata_identity(metadata: &fs::Metadata) -> Result<RegularIdentity, DriverProtocolError> {
    if !metadata.is_file() {
        return Err(DriverProtocolCode::Filesystem.into());
    }
    Ok((0, 0, metadata.len()))
}

fn crate_information_host() -> Vec<String> {
    [
        "/mpk/toolchain/bin/rustc",
        "-",
        "--crate-name",
        "___",
        "--print=file-names",
        "--crate-type",
        "bin",
        "--crate-type",
        "rlib",
        "--crate-type",
        "dylib",
        "--crate-type",
        "cdylib",
        "--crate-type",
        "staticlib",
        "--crate-type",
        "proc-macro",
        "--print=sysroot",
        "--print=split-debuginfo",
        "--print=crate-name",
        "--print=cfg",
        "-Wwarnings",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn crate_information_target(target: &str) -> Vec<String> {
    let mut arguments = crate_information_host();
    arguments.splice(
        5..5,
        [
            "-C",
            "overflow-checks=yes",
            "-C",
            "panic=abort",
            "-C",
            "debug-assertions=no",
            "-C",
            "opt-level=0",
            "-Z",
            "mir-opt-level=0",
            "--remap-path-prefix=/mpk/input=.",
            "--target",
            target,
        ]
        .into_iter()
        .map(str::to_owned),
    );
    arguments
}

fn primary_matches(request: &DriverRequest, arguments: &[String]) -> bool {
    let (_, crate_name, _) = request.selection();
    let target = request.target();
    let expected = [
        "/mpk/toolchain/bin/rustc",
        "--crate-name",
        crate_name,
        "--edition=2021",
        "CRATE_ROOT",
        "--error-format=json",
        "--json=diagnostic-rendered-ansi,artifacts,future-incompat",
        "--crate-type",
        "lib",
        "--emit=dep-info,metadata",
        "-C",
        "embed-bitcode=no",
        "-C",
        "debuginfo=2",
        "--check-cfg",
        "cfg(docsrs,test)",
        "--check-cfg",
        "cfg(feature, values(\"default\"))",
        "-C",
        "METADATA",
        "-C",
        "EXTRA",
        "--out-dir",
        "OUT",
        "--target",
        target,
        "-L",
        "TARGET_DEP",
        "-L",
        "HOST_DEP",
        "-C",
        "overflow-checks=yes",
        "-C",
        "panic=abort",
        "-C",
        "debug-assertions=no",
        "-C",
        "opt-level=0",
        "-Z",
        "mir-opt-level=0",
        "--remap-path-prefix=/mpk/input=.",
    ];
    if arguments.len() != expected.len() {
        return false;
    }
    for (index, (actual, expected)) in arguments.iter().zip(expected).enumerate() {
        let valid = match index {
            4 => request.has_source_path(actual),
            19 => actual.strip_prefix("metadata=").is_some_and(hex16),
            21 => actual.strip_prefix("extra-filename=-").is_some_and(hex16),
            23 => actual == &format!("/mpk/target/{target}/debug/deps"),
            27 => actual == &format!("dependency=/mpk/target/{target}/debug/deps"),
            29 => actual == "dependency=/mpk/target/debug/deps",
            _ => actual == expected,
        };
        if !valid {
            return false;
        }
    }
    true
}

fn primary_with_extra_arguments(request: &DriverRequest, arguments: &[String]) -> bool {
    if arguments.len() > 64 {
        return false;
    }
    [1_usize, 2].into_iter().any(|extra| {
        arguments.len() > extra
            && (0..=arguments.len() - extra).any(|start| {
                let mut candidate = arguments.to_vec();
                candidate.drain(start..start + extra);
                primary_matches(request, &candidate)
            })
    })
}

fn hex16(value: &str) -> bool {
    value.len() == 16
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(target_os = "linux")]
mod platform {
    use std::ffi::{c_int, c_long, CString};
    use std::io;
    use std::os::unix::ffi::OsStrExt;
    use std::path::Path;

    pub(super) const O_NOFOLLOW: c_int = 0o400_000;
    pub(super) const O_DIRECTORY: c_int = 0o200_000;
    const AT_FDCWD: c_int = -100;
    const RENAME_NOREPLACE: u32 = 1;

    const SYS_RENAMEAT2_X86_64: c_long = 316;

    unsafe extern "C" {
        fn syscall(number: c_long, ...) -> c_long;
    }

    pub(super) fn rename_no_replace(old: &Path, new: &Path) -> io::Result<()> {
        let old = CString::new(old.as_os_str().as_bytes())?;
        let new = CString::new(new.as_os_str().as_bytes())?;
        // SAFETY: both paths are live NUL-terminated byte strings and the operation uses the
        // fixed no-replace flag with the current directory namespace.
        if unsafe {
            syscall(
                SYS_RENAMEAT2_X86_64,
                AT_FDCWD,
                old.as_ptr(),
                AT_FDCWD,
                new.as_ptr(),
                RENAME_NOREPLACE,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}

#[cfg(all(unix, not(target_os = "linux")))]
mod platform {
    use std::fs;
    use std::io;
    use std::path::Path;

    pub(super) const O_NOFOLLOW: i32 = 0x100;
    pub(super) const O_DIRECTORY: i32 = 0x100000;

    pub(super) fn rename_no_replace(old: &Path, new: &Path) -> io::Result<()> {
        if fs::symlink_metadata(new).is_ok() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "final exists"));
        }
        fs::rename(old, new)
    }
}
