//! The drifts ENG-34's size check exists to catch: a file past its
//! limit missing from the record of sizes, a listed file grown, a
//! listed file that fits or is gone left in the record, and the check
//! itself no longer naming one of them.

use super::*;

/// The record of sizes the ENG-34 cases drift.
const RECORD: &str = "tooling/engineering/oversize.txt";

/// The check the ENG-34 cases drift.
const CHECK: &str = "tooling/engineering/src/file_sizes.rs";

/// The drifts in the record and in the check against it, late so adding
/// them moves no earlier case to another shard.
pub(super) fn file_size_cases() -> Vec<Case> {
    [record_cases(), check_cases()].concat()
}

/// The record of sizes drifting from the files.
fn record_cases() -> Vec<Case> {
    vec![
        case(
            "a file past its limit dropped from the record of sizes",
            "ENG-34",
            Injection::replace(RECORD, "crates/store/src/lib.rs 2123\n", ""),
            scenario("ENG-34"),
            "crates/store/src/lib.rs holds 2123 lines, past the limit of 1000 for Rust",
        ),
        case(
            "a listed file grown past its recorded size",
            "ENG-34",
            Injection::replace(
                RECORD,
                "features/lane.feature 594",
                "features/lane.feature 590",
            ),
            scenario("ENG-34"),
            "features/lane.feature grew from its recorded 590 lines to 594",
        ),
        case(
            "a file that fits its limit left in the record of sizes",
            "ENG-34",
            Injection::replace(
                RECORD,
                "tooling/testkit/src/boundary/tests.rs 1015",
                "tooling/testkit/src/boundary/tests.rs 1015\ntooling/drift/src/lib.rs 1200",
            ),
            scenario("ENG-34"),
            "tooling/drift/src/lib.rs now fits its limit",
        ),
        case(
            "a file gone from the tree left in the record of sizes",
            "ENG-34",
            Injection::replace(
                RECORD,
                "tooling/testkit/src/boundary/tests.rs 1015",
                "tooling/testkit/src/boundary/tests.rs 1015\ncrates/store/src/gone.rs 1100",
            ),
            scenario("ENG-34"),
            "crates/store/src/gone.rs is listed in tooling/engineering/oversize.txt but is gone",
        ),
    ]
}

/// The size check no longer failing on a drift of the record.
fn check_cases() -> Vec<Case> {
    vec![
        case(
            "the size check accepting a file past its limit that the record does not list",
            "ENG-34",
            Injection::replace(
                CHECK,
                "match record.get(path) {",
                "match record.get(path).or(Some(&usize::MAX)) {",
            ),
            scenario("ENG-34"),
            "the size check does not name crates/unlisted/src/lib.rs",
        ),
        case(
            "the size check accepting a listed file that grew",
            "ENG-34",
            Injection::replace(
                CHECK,
                "Some(&was) if lines > was =>",
                "Some(&was) if lines > was + 1_000_000 =>",
            ),
            scenario("ENG-34"),
            "the size check does not name crates/grown/src/lib.rs",
        ),
        case(
            "the size check keeping a listed file that fits",
            "ENG-34",
            Injection::replace(
                CHECK,
                "Some(&lines) if !past_limit(path, lines) =>",
                "Some(&lines) if lines == usize::MAX =>",
            ),
            scenario("ENG-34"),
            "the size check does not name crates/fits/src/lib.rs",
        ),
        case(
            "the size check keeping a listed file that is gone",
            "ENG-34",
            Injection::replace(
                CHECK,
                "for path in record.keys() {",
                "for path in record.keys().filter(|p| sizes.contains_key(*p)) {",
            ),
            scenario("ENG-34"),
            "the size check does not name crates/gone/src/lib.rs",
        ),
    ]
}
