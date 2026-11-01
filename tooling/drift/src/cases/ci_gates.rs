//! The drifts the list of CI's gates exists to catch (ENG-27): a gate's
//! command written in `ci.yml` as well as in `scripts/ci-gates.txt`, a
//! gate with no job, and a call of a gate the list does not define.

use super::{Case, Injection, case, scenario};

const CI: &str = ".github/workflows/ci.yml";
const CALL: &str = "      - run: scripts/ci-gate.sh toml-lint\n";

pub(super) fn cases() -> Vec<Case> {
    vec![
        case(
            "a gate's command written inline in the CI workflow",
            "ENG-27",
            Injection::replace(CI, CALL, "      - run: taplo lint\n"),
            scenario("ENG-27"),
            "runs the command of gate toml-lint inline",
        ),
        case(
            "a gate that no CI job runs",
            "ENG-27",
            Injection::replace(CI, CALL, ""),
            scenario("ENG-27"),
            "gate toml-lint has no job",
        ),
        case(
            "the CI workflow calling a gate the list does not define",
            "ENG-27",
            Injection::replace(
                CI,
                "scripts/ci-gate.sh markdown\n",
                "scripts/ci-gate.sh markdwon\n",
            ),
            scenario("ENG-27"),
            "calls scripts/ci-gate.sh markdwon, which scripts/ci-gates.txt does not define",
        ),
    ]
}
