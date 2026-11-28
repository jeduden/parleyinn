//! The drifts the evaluation's hold on the files a pull request adds
//! exists to rule out (ENG-32), apart from the registry's other cases
//! so no file of it grows further: a world built without them, an
//! instruction file among them put beside the agent, a link among them
//! followed, one reached through a linked directory copied, and a path
//! among them that leaves the tree written where it points.

use super::case;
use crate::{Case, Injection, cargo_lib_test};

/// The mutations of the copy of a pull request's added files into a
/// world, each run through the unit test that builds one.
pub(super) fn review_world_cases() -> Vec<Case> {
    const ADDED: &str = "tooling/review-eval/src/added.rs";
    vec![
        case(
            "an evaluation world built without the files the evaluated tree adds",
            "ENG-32",
            Injection::replace(
                "tooling/review-eval/src/world.rs",
                "    crate::added::copy(host, tree, &main, &pr_head, &mut notes)?;\n",
                "",
            ),
            cargo_lib_test(
                "review-eval",
                "world::tests::build_puts_the_files_the_evaluated_tree_adds_into_both_trees",
            ),
            "build_puts_the_files_the_evaluated_tree_adds_into_both_trees",
        ),
        case(
            "an instruction file the evaluated tree adds put beside the agent under evaluation",
            "ENG-32",
            Injection::replace(
                ADDED,
                "if !instruction(&rel) {",
                "if !instruction(&rel) || rel.contains('.') {",
            ),
            cargo_lib_test(
                "review-eval",
                "added::tests::an_instruction_file_the_tree_adds_reaches_the_pull_request_s_tree_and_never_main_s",
            ),
            "an_instruction_file_the_tree_adds_reaches_the_pull_request_s_tree_and_never_main_s",
        ),
        case(
            "a link the evaluated tree adds followed into an evaluation world",
            "ENG-32",
            Injection::replace(
                ADDED,
                "tree.join(path).symlink_metadata()",
                "tree.join(path).metadata()",
            ),
            cargo_lib_test(
                "review-eval",
                "added::tests::an_added_path_that_is_no_regular_file_is_left_out_and_noted",
            ),
            "an_added_path_that_is_no_regular_file_is_left_out_and_noted",
        ),
        case(
            "a file the evaluated tree lists through a linked directory copied into an evaluation world",
            "ENG-32",
            Injection::replace(
                ADDED,
                ".all(|dir| is(dir, fs::Metadata::is_dir))",
                ".all(|_| true)",
            ),
            cargo_lib_test(
                "review-eval",
                "added::tests::an_added_path_through_a_linked_directory_is_left_out_and_noted",
            ),
            "an_added_path_through_a_linked_directory_is_left_out_and_noted",
        ),
        case(
            "a path the evaluated tree lists outside itself written where it points",
            "ENG-32",
            Injection::replace(ADDED, "if !inside || ", "if "),
            cargo_lib_test(
                "review-eval",
                "added::tests::an_added_path_that_leaves_the_tree_is_left_out_and_noted",
            ),
            "an_added_path_that_leaves_the_tree_is_left_out_and_noted",
        ),
    ]
}
