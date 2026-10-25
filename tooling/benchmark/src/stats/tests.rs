use super::*;

#[test]
fn twice_median_of_an_odd_count_is_twice_its_middle() {
    assert_eq!(twice_median(&[5, 1, 3]), Some(6));
}

#[test]
fn twice_median_of_an_even_count_sums_its_middle_two() {
    assert_eq!(twice_median(&[4, 1, 3, 2]), Some(5));
}

#[test]
fn twice_median_of_nothing_is_none() {
    assert_eq!(twice_median(&[]), None);
}

#[test]
fn twice_median_holds_the_largest_samples_without_overflow() {
    assert_eq!(
        twice_median(&[u64::MAX, u64::MAX]),
        Some(2 * u128::from(u64::MAX))
    );
}

/// base [1, 2, 3] against head [4, 5, 6]: the head's ranks sum to 15,
/// so U = 15 - 6 = 9 against a mean of 4.5 and a variance of
/// 9 / 12 * 7 = 5.25; with the continuity correction
/// z = (9 - 4.5 - 0.5) / sqrt(5.25).
#[test]
fn mann_whitney_z_matches_the_hand_computed_value() {
    let z = mann_whitney_z(&[1, 2, 3], &[4, 5, 6]);

    assert!((z - 4.0 / 5.25_f64.sqrt()).abs() < 1e-12, "z = {z}");
}

#[test]
fn mann_whitney_z_is_negative_when_the_head_tends_smaller() {
    assert!(mann_whitney_z(&[4, 5, 6], &[1, 2, 3]) < 0.0);
}

/// Ties take their mean rank and shrink the variance: base [1, 2, 2]
/// against head [2, 3, 3]. Ranks: 1 -> 1, the three 2s -> 3, the two
/// 3s -> 5.5, so the head's ranks sum to 3 + 5.5 + 5.5 = 14 and
/// U = 14 - 6 = 8. The tie term is (27 - 3) + (8 - 2) = 30, so the
/// variance is 9 / 12 * (7 - 30 / 30) = 4.5.
#[test]
fn mann_whitney_z_corrects_for_ties() {
    let z = mann_whitney_z(&[1, 2, 2], &[2, 3, 3]);

    assert!((z - 3.0 / 4.5_f64.sqrt()).abs() < 1e-12, "z = {z}");
}

#[test]
fn mann_whitney_z_of_all_tied_samples_is_zero() {
    assert_eq!(mann_whitney_z(&[7, 7], &[7, 7, 7]), 0.0);
}

#[test]
fn mann_whitney_z_of_an_empty_side_is_zero() {
    assert_eq!(mann_whitney_z(&[], &[1, 2]), 0.0);
    assert_eq!(mann_whitney_z(&[1, 2], &[]), 0.0);
}

#[test]
fn stratified_z_of_one_stratum_is_the_mann_whitney_z() {
    let (base, head) = (&[1, 2, 2][..], &[2, 3, 3][..]);
    let plain = mann_whitney_z(base, head);

    let alone = stratified_z(&[(base, head)]);
    let beside_nothing = stratified_z(&[(base, head), (&[], &[7]), (&[7, 7], &[7])]);

    assert!((alone - plain).abs() < 1e-12, "{alone} against {plain}");
    assert!(
        (beside_nothing - plain).abs() < 1e-12,
        "{beside_nothing} against {plain}"
    );
}

/// Three runners of very different speed, the head 12% slower on each.
/// Pooled, the runners' spread hides the slowdown; stratified, each
/// runner's own ranks show it.
#[test]
fn stratified_z_sees_a_slowdown_the_runners_spread_hides() {
    let clean = |ns: u64| (0..10).map(|i| ns + i % 7).collect::<Vec<u64>>();
    let runners: Vec<(Vec<u64>, Vec<u64>)> = [1_000, 3_000, 9_000]
        .into_iter()
        .map(|ns| (clean(ns), clean(ns * 112 / 100)))
        .collect();
    let strata: Vec<(&[u64], &[u64])> = runners
        .iter()
        .map(|(b, h)| (b.as_slice(), h.as_slice()))
        .collect();
    let (base, head): (Vec<u64>, Vec<u64>) =
        runners
            .iter()
            .fold((Vec::new(), Vec::new()), |(mut b, mut h), (rb, rh)| {
                b.extend(rb);
                h.extend(rh);
                (b, h)
            });

    let pooled = mann_whitney_z(&base, &head);
    let stratified = stratified_z(&strata);

    assert!(pooled < crate::gate::Z_CRITICAL, "pooled z = {pooled}");
    assert!(
        stratified > crate::gate::Z_CRITICAL,
        "stratified z = {stratified}"
    );
}

#[test]
fn stratified_z_leaves_out_a_stratum_that_separates_nothing() {
    let (base, head) = (&[1, 2, 3][..], &[4, 5, 6][..]);
    let with_empty = stratified_z(&[(base, head), (&[], &[7, 8])]);
    let with_tied = stratified_z(&[(base, head), (&[7, 7], &[7, 7])]);
    let alone = stratified_z(&[(base, head), (&[], &[])]);

    assert!((with_empty - with_tied).abs() < 1e-12);
    assert!((with_empty - alone).abs() < 1e-12);
    assert!(with_empty > 0.0);
    assert_eq!(stratified_z(&[(&[], &[1]), (&[2, 2], &[2])]), 0.0);
    assert_eq!(stratified_z(&[]), 0.0);
}

#[test]
fn mann_whitney_z_ignores_how_far_an_outlier_lies() {
    let near = mann_whitney_z(&[10, 11, 12, 13], &[14, 15, 16, 17]);
    let far = mann_whitney_z(&[10, 11, 12, 13], &[14, 15, 16, 1_000_000]);

    assert_eq!(near, far);
}
