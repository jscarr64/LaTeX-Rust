//! `Dim` comparison must be exact for every non-NaN value. It multiplied
//! `num * den` with `checked_mul`, so two equal large values (and a value
//! with itself) compared as unordered, `a == a` was false, and a tree holding
//! one never equalled its own reparse (fuzz r16long, `roundtrip-dim-big-eq`).

use latex_rust::{parse, Dim};
use std::cmp::Ordering;

#[test]
fn a_large_dim_equals_itself() {
    let a = Dim::parse("2.1662322688380341817");
    assert!(a.eq_dim(&a));
    assert!(a == a);
    assert_eq!(a.cmp(&a), Some(Ordering::Equal));
    let b = Dim::parse("2.1662322688380341818");
    assert_eq!(a.cmp(&b), Some(Ordering::Less));
    assert_eq!(b.cmp(&a), Some(Ordering::Greater));
    assert_eq!(
        Dim::parse("-2.1662322688380341817").cmp(&a),
        Some(Ordering::Less)
    );
    assert_eq!(
        Dim::parse("-2.1662322688380341817").cmp(&Dim::parse("-2.1662322688380341818")),
        Some(Ordering::Greater)
    );
    assert_eq!(Dim::zero().cmp(&Dim::zero()), Some(Ordering::Equal));
    assert_eq!(Dim::zero().cmp(&a), Some(Ordering::Less));
    assert_eq!(Dim::parse("nan").cmp(&a), None);
}

#[test]
fn the_dim_seeds_round_trip() {
    for name in ["roundtrip-dim-big-eq", "roundtrip-dim-big-eq-b"] {
        let path = format!("{}/fuzz/corpus/parse/{name}", env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let tree = parse(&src).unwrap_or_else(|e| panic!("{name}: {e}"));
        let printed = tree.to_latex();
        assert_eq!(
            parse(&printed).ok().as_ref(),
            Some(&tree),
            "{name}: {printed}"
        );
    }
}

/// The exact comparison agrees with the cross product wherever that fits.
#[test]
fn small_dims_compare_like_cross_products() {
    let vals: Vec<(i64, i64)> = (-9..=9)
        .flat_map(|n| (1..=9).map(move |d| (n * 7, d * 3)))
        .collect();
    for &(n1, d1) in &vals {
        for &(n2, d2) in &vals {
            let want = (n1 * d2).cmp(&(n2 * d1));
            let got = Dim::ratio(n1, d1).cmp(&Dim::ratio(n2, d2));
            assert_eq!(got, Some(want), "{n1}/{d1} vs {n2}/{d2}");
        }
    }
}
