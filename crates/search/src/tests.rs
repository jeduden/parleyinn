use super::*;

fn text(text: &str) -> Request<'_> {
    Request {
        text,
        ..Request::default()
    }
}

fn any_of(terms: &[String]) -> Request<'_> {
    Request {
        any_of: terms,
        ..Request::default()
    }
}

fn expression(request: &Request<'_>) -> String {
    compile(request).unwrap().expression().to_owned()
}

#[test]
fn typed_text_is_literal_terms_all_of_which_must_match() {
    assert_eq!(
        expression(&text("deploy staging")),
        r#""deploy" AND "staging""#
    );
}

#[test]
fn fts5_operators_in_typed_text_are_quoted_words() {
    assert_eq!(
        expression(&text("foo OR bar NEAR(baz)")),
        r#""foo" AND "OR" AND "bar" AND "NEAR(baz)""#
    );
}

#[test]
fn a_quote_in_typed_text_is_doubled_and_ends_nothing() {
    assert_eq!(expression(&text(r#"a"b"#)), r#""a""b""#);
    assert_eq!(
        expression(&text(r#"x" OR "y"#)),
        r#""x""" AND "OR" AND """y""#
    );
}

#[test]
fn a_column_filter_and_a_negation_are_quoted_words() {
    assert_eq!(
        expression(&text("text:secret -hidden")),
        r#""text:secret" AND "-hidden""#
    );
}

#[test]
fn any_whitespace_separates_terms() {
    assert_eq!(
        expression(&text("  a\tb\n c\u{3000}d ")),
        r#""a" AND "b" AND "c" AND "d""#
    );
}

#[test]
fn a_trailing_or_inner_star_is_a_literal_character() {
    assert_eq!(expression(&text("pre* a*b")), r#""pre*" AND "a*b""#);
}

#[test]
fn a_leading_wildcard_is_refused() {
    assert_eq!(compile(&text("*prefix")), Err(Error::LeadingWildcard));
    assert_eq!(compile(&text("ok *")), Err(Error::LeadingWildcard));
}

#[test]
fn sixteen_terms_pass_and_seventeen_are_refused() {
    let sixteen = ["a"; 16].join(" ");
    let seventeen = ["a"; 17].join(" ");

    assert_eq!(compile(&text(&sixteen)).unwrap().terms(), 16);
    assert_eq!(compile(&text(&seventeen)), Err(Error::TooManyTerms));
}

#[test]
fn the_terms_of_every_parameter_count_together() {
    let any_of: Vec<String> = ["a", "b", "c"].map(str::to_owned).into();
    let all_of: Vec<String> = ["d", "e"].map(str::to_owned).into();
    let twelve = ["t"; 12].join(" ");
    let request = Request {
        text: &twelve,
        any_of: &any_of,
        all_of: &all_of,
        phrase: Some("p q"),
    };

    assert_eq!(compile(&request), Err(Error::TooManyTerms));
}

#[test]
fn a_huge_text_is_refused_without_splitting_all_of_it() {
    let huge = "a ".repeat(1 << 20);

    assert_eq!(compile(&text(&huge)), Err(Error::TooManyTerms));
}

#[test]
fn a_term_of_64_characters_passes_and_one_of_65_is_refused() {
    let at = "é".repeat(64);
    let over = "é".repeat(65);

    assert!(compile(&text(&at)).is_ok());
    assert_eq!(compile(&text(&over)), Err(Error::TermTooLong));
}

#[test]
fn a_control_character_is_refused() {
    assert_eq!(compile(&text("a\u{0}b")), Err(Error::Control));
    assert_eq!(compile(&text("a\u{7}")), Err(Error::Control));
}

#[test]
fn a_term_with_no_letter_or_digit_is_left_out_but_counted() {
    let compiled = compile(&text("deploy --- ???")).unwrap();

    assert_eq!(compiled.expression(), r#""deploy""#);
    assert_eq!(compiled.terms(), 3);
}

#[test]
fn a_query_with_no_searchable_term_is_refused() {
    assert_eq!(compile(&text("")), Err(Error::Empty));
    assert_eq!(compile(&text("   ")), Err(Error::Empty));
    assert_eq!(compile(&text("--- ...")), Err(Error::Empty));
}

#[test]
fn any_of_is_an_or_group() {
    let any_of: Vec<String> = ["red", "green"].map(str::to_owned).into();
    let request = Request {
        text: "paint",
        any_of: &any_of,
        ..Request::default()
    };

    assert_eq!(expression(&request), r#""paint" AND ("red" OR "green")"#);
}

#[test]
fn one_any_of_term_needs_no_group() {
    let any_of: Vec<String> = vec!["red".to_owned()];
    let request = Request {
        any_of: &any_of,
        ..Request::default()
    };

    assert_eq!(expression(&request), r#""red""#);
}

#[test]
fn all_of_terms_are_each_required() {
    let all_of: Vec<String> = ["red", "green"].map(str::to_owned).into();
    let request = Request {
        all_of: &all_of,
        ..Request::default()
    };

    assert_eq!(expression(&request), r#""red" AND "green""#);
}

#[test]
fn a_structured_term_may_hold_spaces_and_is_one_phrase() {
    let all_of: Vec<String> = vec!["red car".to_owned()];
    let request = Request {
        all_of: &all_of,
        ..Request::default()
    };

    assert_eq!(expression(&request), r#""red car""#);
    assert_eq!(compile(&request).unwrap().terms(), 1);
}

#[test]
fn a_phrase_is_one_quoted_run_of_words() {
    let request = Request {
        phrase: Some(r#"exact "words""#),
        ..Request::default()
    };

    assert_eq!(expression(&request), r#""exact ""words""""#);
}

#[test]
fn a_structured_term_obeys_the_same_limits() {
    let long = vec!["x".repeat(65)];
    let star = vec!["*x".to_owned()];
    let control = vec!["a\nb".to_owned()];
    assert_eq!(compile(&any_of(&long)), Err(Error::TermTooLong));
    assert_eq!(compile(&any_of(&star)), Err(Error::LeadingWildcard));
    assert_eq!(compile(&any_of(&control)), Err(Error::Control));
}

#[test]
fn an_empty_structured_term_is_left_out() {
    let all_of: Vec<String> = vec![String::new(), "ok".to_owned()];
    let request = Request {
        all_of: &all_of,
        ..Request::default()
    };

    assert_eq!(expression(&request), r#""ok""#);
}

#[test]
fn a_refusal_is_fixed_text_that_repeats_nothing() {
    let shown: Vec<String> = [
        Error::Empty,
        Error::TooManyTerms,
        Error::TermTooLong,
        Error::LeadingWildcard,
        Error::Control,
    ]
    .iter()
    .map(ToString::to_string)
    .collect();

    assert_eq!(
        shown,
        [
            "the query holds no searchable term",
            "more than 16 terms",
            "term longer than 64 characters",
            "leading wildcard",
            "a term holds a control character",
        ]
    );
}
