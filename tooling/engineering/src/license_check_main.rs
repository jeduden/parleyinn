//! The `license-check` command checks that every crate a release links
//! carries a license text that fits its declared license (ENG-30) and
//! that each target's committed `THIRD_PARTY_LICENSES` equals its
//! regeneration (ENG-31). CI runs it on every pull request. It reads the
//! checkout and the sources cargo has fetched, and writes the terminal; it writes files only when
//! asked to, with `--write` and `--out`. It is CI tooling and never ships.

use std::io;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let code = engineering::license_texts::cli::run(
        std::env::args().skip(1),
        &mut io::stdout(),
        &mut io::stderr(),
        &|root, options| engineering::third_party::run_release(&cargo, root, options),
    );

    ExitCode::from(code)
}
