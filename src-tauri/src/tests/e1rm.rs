use crate::dao::set::Set;

#[test]
fn single_rep_is_the_lifted_weight() {
    assert_eq!(Set::estimate_1rm(100.0, 0), 100);
    assert_eq!(Set::estimate_1rm(100.0, 1), 100);
}

#[test]
fn brzycki_up_to_ten_reps() {
    // weight * 36 / (37 - reps)
    assert_eq!(Set::estimate_1rm(100.0, 5), 112);
    assert_eq!(Set::estimate_1rm(100.0, 10), 133);
}

#[test]
fn eleven_to_twenty_reps() {
    // 100 * weight / (101.3 - 2.67123 * reps)
    assert_eq!(Set::estimate_1rm(100.0, 11), 139);
    assert_eq!(Set::estimate_1rm(100.0, 20), 208);
}

#[test]
fn zero_weight_is_zero() {
    for reps in 0..=40 {
        assert_eq!(Set::estimate_1rm(0.0, reps), 0, "reps = {reps}");
    }
}

#[test]
fn never_below_lifted_weight() {
    for reps in 1..=40 {
        assert!(
            Set::estimate_1rm(100.0, reps) >= 100,
            "reps = {reps} -> {}",
            Set::estimate_1rm(100.0, reps)
        );
    }
}

/// More reps with the same weight must never yield a lower estimate, otherwise PR
/// detection and progress charts show a drop when the athlete actually improved.
#[test]
fn non_decreasing_with_reps() {
    let mut previous = Set::estimate_1rm(100.0, 1);
    for reps in 2..=40 {
        let current = Set::estimate_1rm(100.0, reps);
        assert!(
            current >= previous,
            "e1RM drops from {previous} ({} reps) to {current} ({reps} reps)",
            reps - 1
        );
        previous = current;
    }
}
