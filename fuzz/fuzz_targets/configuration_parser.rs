//! ENG-07, ADM-04: the configuration parser on hostile settings
//! layers. Each input is the text of a settings layer, managed policy,
//! the principal's configuration or a repository's `.parinn.toml`; the
//! parser must never panic, only accept it or name what fails.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };
    let _ = configuration::layer::parse(text);
});
