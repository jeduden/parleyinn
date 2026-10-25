//! `parinn licenses` (ENG-31): the third-party license texts compiled
//! into this executable. The text is the committed
//! `licenses/<target-triple>/THIRD_PARTY_LICENSES` of the release target
//! the executable is built for, chosen below by the build's operating
//! system and architecture. It is a string in the executable: the command
//! reads no file, opens no store and no socket. The repository's
//! `license-check` regenerates each target's file and fails a pull
//! request when it differs, and the release build compares this
//! command's output on each platform with the regeneration.

use std::io::Write;

use crate::{EXIT_USAGE, written};

/// The third-party licenses of this build's release target. A build for
/// a target that is none of the release targets gets the file of the
/// release target on its operating system and architecture, so a
/// development build answers too.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES");

/// The third-party licenses of this build's release target.
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../licenses/aarch64-unknown-linux-musl/THIRD_PARTY_LICENSES");

/// The third-party licenses of this build's release target.
#[cfg(target_os = "windows")]
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../licenses/x86_64-pc-windows-gnu/THIRD_PARTY_LICENSES");

/// The third-party licenses of this build's release target.
#[cfg(target_os = "macos")]
pub const THIRD_PARTY_LICENSES: &str =
    include_str!("../../../licenses/aarch64-apple-darwin/THIRD_PARTY_LICENSES");

/// The committed file the build above embeds, relative to the
/// repository, for the test that compares the two.
#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
pub const EMBEDDED_FILE: &str = "licenses/x86_64-unknown-linux-musl/THIRD_PARTY_LICENSES";

/// The committed file this build embeds.
#[cfg(all(test, target_os = "linux", target_arch = "aarch64"))]
pub const EMBEDDED_FILE: &str = "licenses/aarch64-unknown-linux-musl/THIRD_PARTY_LICENSES";

/// The committed file this build embeds.
#[cfg(all(test, target_os = "windows"))]
pub const EMBEDDED_FILE: &str = "licenses/x86_64-pc-windows-gnu/THIRD_PARTY_LICENSES";

/// The committed file this build embeds.
#[cfg(all(test, target_os = "macos"))]
pub const EMBEDDED_FILE: &str = "licenses/aarch64-apple-darwin/THIRD_PARTY_LICENSES";

/// `parinn licenses`: prints the embedded texts, byte for byte. It takes
/// no flag.
pub(crate) fn licenses(
    mut args: impl Iterator<Item = String>,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> u8 {
    if let Some(arg) = args.next() {
        let _ = writeln!(stderr, "parinn licenses: unknown argument {arg:?}");
        return EXIT_USAGE;
    }

    written(stdout.write_all(THIRD_PARTY_LICENSES.as_bytes()), stderr)
}

#[cfg(test)]
mod tests;
