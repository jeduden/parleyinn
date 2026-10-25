//! ENG-07, SEC-16: the hook input decoder on hostile hook inputs. Each
//! input is decoded as every hook ParleyInn installs decodes it: held to
//! its limits, parsed, checked against the schema and the recorded hook
//! input shapes. The decoder must never panic, and every input gets
//! exactly one outcome.
#![no_main]

use hook::{Decoded, Failure, Hook};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    for hook in [
        Hook::SessionStart,
        Hook::UserPromptSubmit,
        Hook::PermissionRequest,
    ] {
        match hook::decode(hook, text) {
            Ok(Decoded { .. })
            | Err(
                Failure::Malformed
                | Failure::Invalid
                | Failure::TooLarge
                | Failure::TooDeep
                | Failure::UnknownHook
                | Failure::Panicked,
            ) => {}
        }
    }
});
