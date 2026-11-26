//! A minimal repository tree for the tests that read one from disk.

use std::fs;
use std::path::Path;

/// Three requirements, each a block.
pub const SRS: &str = "# 5\n\n## REC-01\n\n- Pri: P0\n- Ver: T\n- Traces: I1\n\nIngest.\n\n## ENG-11\n\n- Pri: P0\n- Ver: A\n- Traces: I1\n\nCoverage for security-sensitive crates (`pin`).\n\n## ENG-29\n\n- Pri: P0\n- Ver: I\n- Traces: I2\n\nInvariants and PRV-10 need an ADR.\n";

/// One live scenario and one pending.
pub const FEATURES: &str = "Feature: Record\n\n  @REC-01 @P0 @I1\n  Scenario: ingest\n    Given a transcript\n\n  @ENG-29 @P0 @I2 @pending\n  Scenario: guarded\n    Given nothing\n";

/// Writes the tree under `root`: the SRS, the features, the invariants,
/// CODEOWNERS and one decision record.
pub fn tree(root: &Path) {
    let write = |rel: &str, body: &str| {
        let path = root.join(rel);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(path, body).unwrap();
    };
    write("docs/srs/05.md", SRS);
    write("docs/srs/invariants.md", "I1 holds.\n");
    write("features/record.feature", FEATURES);
    write(".github/CODEOWNERS", "/docs/srs/ @owner\n");
    write(
        "docs/adr/ADR-2610111200-x.md",
        "---\nid: ADR-2610111200\ntitle: \"x\"\nstatus: accepted\nsummary: >-\n  s\n---\n# ADR-2610111200: x\n",
    );
    write("docs/adr/notes.txt", "not a record");
}
