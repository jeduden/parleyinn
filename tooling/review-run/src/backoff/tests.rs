use std::time::Duration;

use super::*;

#[test]
fn an_overloaded_or_rate_limited_run_waits() {
    for (failure, stderr) in [
        (
            r#"its run ended with error: API Error: 529 {"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#,
            "",
        ),
        (
            r#"its run ended with error: API Error: 429 {"type":"error","error":{"type":"rate_limit_error"}}"#,
            "",
        ),
        (
            "its run ended with error: Repeated 529 Overloaded errors",
            "",
        ),
        ("its run ended with none", "API returned status 529\n"),
        (
            "its run answered nothing",
            "Rate limit exceeded, retry later\n",
        ),
    ] {
        assert_eq!(class(failure, stderr), Class::Overloaded, "{failure}");
    }
}

#[test]
fn a_run_a_cost_cap_stopped_is_never_run_again() {
    for failure in [
        "its run ended with error_max_budget_usd",
        "over budget: the review's runs cost 40.0000 USD, its cap is 40.00 USD, so it was not started",
    ] {
        assert_eq!(
            class(failure, "overloaded"),
            Class::Never("a cost cap stopped it, and another run would cost more"),
            "{failure}"
        );
    }
}

#[test]
fn a_run_whose_credential_was_refused_is_never_run_again() {
    for (failure, stderr) in [
        (
            r#"its run ended with error: API Error: 401 {"type":"error","error":{"type":"authentication_error","message":"invalid token"}}"#,
            "",
        ),
        ("its run ended with error: API Error: 403 forbidden", ""),
        (
            "its run ended with none",
            "Invalid API key · Please run /login\n",
        ),
        ("its run ended with none", "OAuth: Invalid token\n"),
        ("its run ended with none", "status 401 and overloaded\n"),
    ] {
        assert_eq!(
            class(failure, stderr),
            Class::Never("its credential was refused, and no run with it can succeed"),
            "{failure} {stderr}"
        );
    }
}

#[test]
fn any_other_failure_runs_once_more() {
    for failure in [
        "its run ended with error_max_turns",
        "its run ended with deadline",
        "its run answered nothing",
        "start claude: No such file or directory",
    ] {
        assert_eq!(class(failure, ""), Class::Again, "{failure}");
    }
}

#[test]
fn each_wait_doubles_with_up_to_half_again_as_jitter() {
    let left = Duration::from_secs(900);

    assert_eq!(wait(0, 0.0, left), Some(Duration::from_secs(15)));
    assert_eq!(wait(1, 0.0, left), Some(Duration::from_secs(30)));
    assert_eq!(wait(0, 0.5, left), Some(Duration::from_millis(18_750)));
    assert_eq!(wait(1, 1.5, left), Some(Duration::from_secs(45)));
    assert_eq!(wait(0, -1.0, left), Some(Duration::from_secs(15)));
}

#[test]
fn a_wait_is_cut_to_what_is_left_of_the_deadline() {
    assert_eq!(
        wait(1, 0.9, Duration::from_secs(20)),
        Some(Duration::from_secs(20))
    );
    assert_eq!(wait(0, 0.0, Duration::ZERO), None);
    assert_eq!(
        wait(40, 0.0, Duration::from_secs(5)),
        Some(Duration::from_secs(5))
    );
}
