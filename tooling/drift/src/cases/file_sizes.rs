//! The drifts ENG-34's size check exists to catch: a file past its
//! limit missing from the size list, a listed file grown, a listed
//! file that shrank whose listed lines were not lowered, a listed file
//! that fits or is gone left in the size list, and the check itself no
//! longer naming one of them.

use super::*;

/// The size list the ENG-34 cases drift.
const SIZE_LIST: &str = "tooling/engineering/oversize.txt";

/// The check the ENG-34 cases drift.
const CHECK: &str = "tooling/engineering/src/file_sizes.rs";

/// The drifts in the size list and in the check against it, late so adding
/// them moves no earlier case to another shard.
pub(super) fn file_size_cases() -> Vec<Case> {
    [size_list_cases(), check_cases()].concat()
}

/// The size list drifting from the files.
fn size_list_cases() -> Vec<Case> {
    vec![
        case(
            "a file past its limit dropped from the size list",
            "ENG-34",
            Injection::replace(SIZE_LIST, "crates/store/src/lib.rs 2123\n", ""),
            scenario("ENG-34"),
            "crates/store/src/lib.rs holds 2123 lines, past the limit of 1000 for Rust",
        ),
        case(
            "a listed file grown past its listed size",
            "ENG-34",
            Injection::replace(
                SIZE_LIST,
                "features/lane.feature 594",
                "features/lane.feature 590",
            ),
            scenario("ENG-34"),
            "features/lane.feature grew from its listed 590 lines to 594",
        ),
        case(
            "a listed file shrunk whose listed lines were not lowered",
            "ENG-34",
            Injection::replace(
                SIZE_LIST,
                "features/lane.feature 594",
                "features/lane.feature 598",
            ),
            scenario("ENG-34"),
            "features/lane.feature holds 594 lines, fewer than its listed 598",
        ),
        case(
            "a file that fits its limit left in the size list",
            "ENG-34",
            Injection::replace(
                SIZE_LIST,
                "tooling/testkit/src/boundary/tests.rs 1015",
                "tooling/testkit/src/boundary/tests.rs 1015\ntooling/drift/src/lib.rs 1200",
            ),
            scenario("ENG-34"),
            "tooling/drift/src/lib.rs now fits its limit",
        ),
        case(
            "a file gone from the tree left in the size list",
            "ENG-34",
            Injection::replace(
                SIZE_LIST,
                "tooling/testkit/src/boundary/tests.rs 1015",
                "tooling/testkit/src/boundary/tests.rs 1015\ncrates/store/src/gone.rs 1100",
            ),
            scenario("ENG-34"),
            "crates/store/src/gone.rs is listed in tooling/engineering/oversize.txt but is gone",
        ),
    ]
}

/// The size check no longer failing on a drift of the size list.
fn check_cases() -> Vec<Case> {
    vec![
        case(
            "the size check accepting a file past its limit that the size list does not list",
            "ENG-34",
            Injection::replace(
                CHECK,
                "match listed.get(path) {",
                "match listed.get(path).or(Some(&usize::MAX)) {",
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
            "the size check accepting a listed file that shrank",
            "ENG-34",
            Injection::replace(
                CHECK,
                "Some(&lines) if lines < was &&",
                "Some(&lines) if lines + 1_000_000 < was &&",
            ),
            scenario("ENG-34"),
            "the size check does not name crates/shrunk/src/lib.rs",
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
                "for (path, &was) in listed {",
                "for (path, &was) in listed.iter().filter(|(p, _)| sizes.contains_key(*p)) {",
            ),
            scenario("ENG-34"),
            "the size check does not name crates/gone/src/lib.rs",
        ),
    ]
}
