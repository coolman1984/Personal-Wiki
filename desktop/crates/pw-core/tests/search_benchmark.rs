//! Search performance at scale.
//!
//! Builds a ~10,000 item database and measures typical queries. The target in
//! docs/UI_SPEC.md is under 150 ms for a normal local query; this test asserts a
//! loose ceiling (so it cannot flake on a busy CI box) and prints the real
//! numbers, which is what the completion report quotes.

mod support;
use pw_core::models::SearchFilters;
use pw_core::{apply, pack, search, staging, Store};
use std::time::Instant;

const TARGET_ITEMS: usize = 10_000;

fn build_large_store() -> Store {
    let mut store = support::store();
    let words = [
        "session",
        "agent",
        "context",
        "install",
        "token",
        "branch",
        "review",
        "model",
        "permission",
        "sandbox",
        "worktree",
        "prompt",
        "index",
        "backup",
        "diagnose",
    ];
    let arabic = [
        "جلسة",
        "وكيل",
        "سياق",
        "تثبيت",
        "رمز",
        "فرع",
        "مراجعة",
        "نموذج",
        "صلاحية",
        "فهرس",
    ];

    let mut items = Vec::with_capacity(TARGET_ITEMS);
    for i in 0..TARGET_ITEMS {
        let w = words[i % words.len()];
        let a = arabic[i % arabic.len()];
        items.push(format!(
            r#"{{"externalKey":"bench:item-{i}","slug":"bench-item-{i}","toolSlug":"claude-code",
                "itemType":"cli_command","title":"bench {w} {i}",
                "shortDescription":"أمر {a} رقم {i} لاختبار الأداء.",
                "syntax":"bench {w} --id {i}","verificationStatus":"demo",
                "aliases":["{a} {i}","bench alias {i}"],
                "tasks":["search"],"tags":["cli"]}}"#
        ));
    }
    let json = support::pack_json(&items.join(","), "");
    let parsed = pack::parse_and_validate(&json, Some(&store)).expect("large pack validates");
    let batch = staging::stage_pack(&store, &parsed, "bench").unwrap();
    staging::set_review_status(&store, &batch, None, "approved").unwrap();
    apply::apply_batch(&mut store, &batch, "bench").unwrap();
    store
}

#[test]
fn search_stays_fast_with_ten_thousand_items() {
    let build_start = Instant::now();
    let store = build_large_store();
    let build_ms = build_start.elapsed().as_secs_f64() * 1000.0;

    let total = store.count("items").unwrap();
    assert!(
        total >= TARGET_ITEMS as i64,
        "expected a large database, got {total}"
    );
    let (status, indexed) = search::health(store.conn());
    assert_eq!(status, "ok");
    assert_eq!(indexed, total);

    let queries = [
        "session",          // common latin term, thousands of matches
        "bench agent 4242", // narrow multi-token
        "جلسة",             // common arabic term
        "وكيل 777",         // arabic + digits
        "bench alias 9999", // alias-only match
    ];

    let mut worst: f64 = 0.0;
    println!("\n--- search benchmark ({total} items, index built in {build_ms:.0} ms) ---");
    for query in queries {
        // one warm-up, then three measured runs
        let _ = search::search(&store, query, &SearchFilters::default(), 25, 0).unwrap();
        let mut best = f64::MAX;
        let mut hits = 0;
        for _ in 0..3 {
            let start = Instant::now();
            let response = search::search(&store, query, &SearchFilters::default(), 25, 0).unwrap();
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            best = best.min(ms);
            hits = response.total;
        }
        worst = worst.max(best);
        println!("  {query:<20} {best:>7.1} ms   {hits} match(es)");
    }
    println!("--- worst query: {worst:.1} ms ---\n");

    // Loose ceiling: the point is to catch an order-of-magnitude regression,
    // not to fail because a CI runner was busy. The measured numbers are printed
    // above and quoted in the completion report.
    assert!(
        worst < 1500.0,
        "search regressed badly: worst query took {worst:.1} ms"
    );
}
