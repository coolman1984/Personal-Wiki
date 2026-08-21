mod support;

#[test]
fn fresh_database_migrates_and_seeds_through_the_real_pipeline() {
    let store = support::store();
    assert_eq!(store.schema_version().unwrap(), pw_core::latest_schema_version());
    assert_eq!(store.integrity_check().unwrap(), "ok");
    assert_eq!(store.foreign_key_check().unwrap(), 0);

    let tools = pw_core::repo::tool_cards(&store).unwrap();
    assert!(tools.len() >= 8, "expected the starter tool families, got {}", tools.len());

    // The seed went through stage → approve → apply for the taxonomy pack and
    // each content pack, so it left a real audit trail — not a shortcut.
    let batches = pw_core::staging::list_batches(&store, 10).unwrap();
    assert_eq!(batches.len(), pw_core::seed::STARTER_PACKS.len());
    assert!(batches.iter().all(|b| b.status == "applied"));
}
