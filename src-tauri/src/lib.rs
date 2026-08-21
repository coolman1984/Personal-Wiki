//! Personal Wiki desktop shell.
//!
//! Every command here is a thin, typed wrapper over `pw-core`. The frontend has
//! no database access of its own, cannot execute anything, and cannot reach the
//! filesystem except through the single Knowledge Pack picker below.

use pw_core::models::*;
use pw_core::{apply, audit, backup, diagnostics, edit, pack, paths, personal, repo, search, seed, staging, url as safe_url, Store};
use serde::Serialize;
use std::sync::Mutex;
use tauri::{Manager, State};

pub struct AppState {
    store: Mutex<Store>,
}

/// Errors cross the IPC boundary as readable strings; validation issues keep
/// their field paths so the UI can show exactly what is wrong.
fn to_message(err: pw_core::CoreError) -> String {
    match err {
        pw_core::CoreError::Validation(issues) => {
            let details = issues
                .iter()
                .take(8)
                .map(|issue| format!("{} — {}", issue.path, issue.message))
                .collect::<Vec<_>>()
                .join("\n");
            format!("الحزمة غير صالحة ({} مشكلة):\n{details}", issues.len())
        }
        other => other.to_string(),
    }
}

type CmdResult<T> = Result<T, String>;

fn with_store<T, F>(state: &State<AppState>, action: F) -> CmdResult<T>
where
    F: FnOnce(&Store) -> pw_core::Result<T>,
{
    let store = state.store.lock().map_err(|_| "قاعدة البيانات مشغولة".to_string())?;
    action(&store).map_err(to_message)
}

fn with_store_mut<T, F>(state: &State<AppState>, action: F) -> CmdResult<T>
where
    F: FnOnce(&mut Store) -> pw_core::Result<T>,
{
    let mut store = state.store.lock().map_err(|_| "قاعدة البيانات مشغولة".to_string())?;
    action(&mut store).map_err(to_message)
}

// ---------------------------------------------------------------- browse

#[tauri::command]
fn home_summary(state: State<AppState>) -> CmdResult<HomeSummary> {
    with_store(&state, repo::home_summary)
}

#[tauri::command]
fn list_tools(state: State<AppState>) -> CmdResult<Vec<ToolCard>> {
    with_store(&state, repo::tool_cards)
}

#[tauri::command]
fn tool_page(state: State<AppState>, slug: String) -> CmdResult<ToolPage> {
    with_store(&state, |store| repo::tool_page(store, &slug))
}

#[tauri::command]
fn list_tasks(state: State<AppState>) -> CmdResult<Vec<TaskCard>> {
    with_store(&state, repo::task_cards)
}

#[tauri::command]
fn category_tree(state: State<AppState>) -> CmdResult<Vec<CategoryNode>> {
    with_store(&state, |store| repo::category_tree(store, None))
}

#[tauri::command]
fn category_path(state: State<AppState>, slug: String) -> CmdResult<Vec<Category>> {
    with_store(&state, |store| repo::category_path(store, &slug))
}

#[tauri::command]
fn task_detail(state: State<AppState>, slug: String) -> CmdResult<Task> {
    with_store(&state, |store| repo::task_by_slug(store, &slug))
}

#[tauri::command]
fn category_detail(state: State<AppState>, slug: String) -> CmdResult<Category> {
    with_store(&state, |store| repo::category_by_slug(store, &slug))
}

#[tauri::command]
fn items_for_tool(
    state: State<AppState>,
    slug: String,
    section: Option<String>,
    sort: Option<String>,
) -> CmdResult<Vec<ItemSummary>> {
    with_store(&state, |store| {
        repo::items_for_tool(store, &slug, section.as_deref(), repo::ItemSort::parse(sort.as_deref()), 500)
    })
}

#[tauri::command]
fn items_for_task(state: State<AppState>, slug: String, sort: Option<String>) -> CmdResult<Vec<ItemSummary>> {
    with_store(&state, |store| {
        repo::items_for_task(store, &slug, repo::ItemSort::parse(sort.as_deref()), 500)
    })
}

#[tauri::command]
fn items_for_category(state: State<AppState>, slug: String, sort: Option<String>) -> CmdResult<Vec<ItemSummary>> {
    with_store(&state, |store| {
        repo::items_for_category(store, &slug, repo::ItemSort::parse(sort.as_deref()), 500)
    })
}

#[tauri::command]
fn item_detail(state: State<AppState>, slug: String) -> CmdResult<ItemDetail> {
    with_store(&state, |store| repo::item_detail(store, &slug))
}

#[tauri::command]
fn record_open(state: State<AppState>, item_id: String) -> CmdResult<()> {
    with_store(&state, |store| personal::record_open(store, &item_id))
}

#[tauri::command]
fn list_tags(state: State<AppState>) -> CmdResult<Vec<Tag>> {
    with_store(&state, repo::list_tags)
}

#[tauri::command]
fn list_item_types(state: State<AppState>) -> CmdResult<Vec<String>> {
    with_store(&state, repo::list_item_types)
}

#[tauri::command]
fn list_favorites(state: State<AppState>) -> CmdResult<Vec<ItemSummary>> {
    with_store(&state, repo::favorites)
}

#[tauri::command]
fn list_pinned(state: State<AppState>) -> CmdResult<Vec<ItemSummary>> {
    with_store(&state, repo::pinned)
}

// ---------------------------------------------------------------- search

#[tauri::command]
fn search(
    state: State<AppState>,
    query: String,
    filters: Option<SearchFilters>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> CmdResult<SearchResponse> {
    let filters = filters.unwrap_or_default();
    with_store(&state, |store| {
        let response = search::search(store, &query, &filters, limit.unwrap_or(25), offset.unwrap_or(0))?;
        personal::record_search(store, &query, &filters, response.total)?;
        Ok(response)
    })
}

#[tauri::command]
fn search_suggestions(state: State<AppState>, query: String) -> CmdResult<Vec<String>> {
    with_store(&state, |store| search::suggestions(store, &query, 6))
}

#[tauri::command]
fn search_history(state: State<AppState>) -> CmdResult<Vec<SearchHistoryEntry>> {
    with_store(&state, |store| personal::search_history(store, 12))
}

#[tauri::command]
fn clear_search_history(state: State<AppState>) -> CmdResult<()> {
    with_store(&state, personal::clear_search_history)
}

#[tauri::command]
fn list_saved_searches(state: State<AppState>) -> CmdResult<Vec<SavedSearch>> {
    with_store(&state, personal::list_saved_searches)
}

#[tauri::command]
fn save_search(
    state: State<AppState>,
    name: String,
    query: String,
    filters: Option<SearchFilters>,
) -> CmdResult<SavedSearch> {
    let filters = filters.unwrap_or_default();
    with_store(&state, |store| personal::save_search(store, &name, &query, &filters))
}

#[tauri::command]
fn delete_saved_search(state: State<AppState>, id: String) -> CmdResult<()> {
    with_store(&state, |store| personal::delete_saved_search(store, &id))
}

// ---------------------------------------------------------------- personal layer

#[tauri::command]
fn set_favorite(state: State<AppState>, item_id: String, favorite: bool) -> CmdResult<bool> {
    with_store(&state, |store| personal::set_favorite(store, &item_id, favorite))
}

#[tauri::command]
fn set_pin(state: State<AppState>, item_id: String, pinned: bool) -> CmdResult<bool> {
    with_store(&state, |store| personal::set_pin(store, &item_id, pinned))
}

#[tauri::command]
fn list_notes(state: State<AppState>) -> CmdResult<Vec<Note>> {
    with_store(&state, personal::list_notes)
}

#[tauri::command]
fn save_note(
    state: State<AppState>,
    id: Option<String>,
    item_id: Option<String>,
    title: String,
    body: String,
) -> CmdResult<Note> {
    with_store(&state, |store| {
        personal::save_note(store, id.as_deref(), item_id.as_deref(), &title, &body)
    })
}

#[tauri::command]
fn delete_note(state: State<AppState>, id: String) -> CmdResult<()> {
    with_store(&state, |store| personal::delete_note(store, &id))
}

#[tauri::command]
fn list_links(state: State<AppState>) -> CmdResult<Vec<SavedLink>> {
    with_store(&state, personal::list_links)
}

#[allow(clippy::too_many_arguments)]
#[tauri::command]
fn save_link(
    state: State<AppState>,
    id: Option<String>,
    title: String,
    url: String,
    description: Option<String>,
    tool_id: Option<String>,
    category_id: Option<String>,
    is_favorite: Option<bool>,
) -> CmdResult<SavedLink> {
    with_store(&state, |store| {
        personal::save_link(
            store,
            id.as_deref(),
            &title,
            &url,
            description.as_deref(),
            tool_id.as_deref(),
            category_id.as_deref(),
            is_favorite.unwrap_or(false),
        )
    })
}

#[tauri::command]
fn delete_link(state: State<AppState>, id: String) -> CmdResult<()> {
    with_store(&state, |store| personal::delete_link(store, &id))
}

// ---------------------------------------------------------------- editing

#[tauri::command]
fn save_tool(state: State<AppState>, input: edit::ToolInput) -> CmdResult<Tool> {
    with_store(&state, |store| edit::save_tool(store, input))
}

#[tauri::command]
fn save_task(state: State<AppState>, input: edit::TaskInput) -> CmdResult<Task> {
    with_store(&state, |store| edit::save_task(store, input))
}

#[tauri::command]
fn save_category(state: State<AppState>, input: edit::CategoryInput) -> CmdResult<Category> {
    with_store(&state, |store| edit::save_category(store, input))
}

#[tauri::command]
fn save_item(state: State<AppState>, input: edit::ItemInput) -> CmdResult<ItemDetail> {
    with_store(&state, |store| edit::save_item(store, input))
}

#[tauri::command]
fn set_active(state: State<AppState>, entity: String, id: String, active: bool) -> CmdResult<()> {
    with_store(&state, |store| edit::set_active(store, &entity, &id, active))
}

// ---------------------------------------------------------------- updates

#[tauri::command]
fn list_sources(state: State<AppState>) -> CmdResult<Vec<Source>> {
    with_store(&state, repo::list_sources)
}

#[tauri::command]
fn list_batches(state: State<AppState>) -> CmdResult<Vec<UpdateBatch>> {
    with_store(&state, |store| staging::list_batches(store, 50))
}

#[tauri::command]
fn list_changes(
    state: State<AppState>,
    batch_id: Option<String>,
    status: Option<String>,
) -> CmdResult<Vec<StagedChange>> {
    with_store(&state, |store| staging::list_changes(store, batch_id.as_deref(), status.as_deref()))
}

#[tauri::command]
fn review_changes(
    state: State<AppState>,
    batch_id: String,
    change_ids: Option<Vec<String>>,
    status: String,
) -> CmdResult<usize> {
    with_store(&state, |store| {
        staging::set_review_status(store, &batch_id, change_ids.as_deref(), &status)
    })
}

#[tauri::command]
fn apply_batch(state: State<AppState>, batch_id: String) -> CmdResult<ApplyOutcome> {
    with_store_mut(&state, |store| apply::apply_batch(store, &batch_id, "user"))
}

#[tauri::command]
fn revert_batch(state: State<AppState>, batch_id: String) -> CmdResult<ApplyOutcome> {
    with_store_mut(&state, |store| apply::revert_batch(store, &batch_id, "user"))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOutcome {
    batch_id: String,
    staged: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidateOutcome {
    items: usize,
    producer: String,
}

/// Import a Knowledge Pack the user picked. Staging is the default; applying is
/// a separate, explicit decision made in the Update Center.
#[tauri::command]
fn import_pack_file(state: State<AppState>, path: String, apply: Option<bool>) -> CmdResult<ImportOutcome> {
    let json = std::fs::read_to_string(&path).map_err(|e| format!("تعذّرت قراءة الملف: {e}"))?;
    with_store_mut(&state, |store| {
        let parsed = pack::parse_and_validate(&json, Some(store))?;
        let batch_id = staging::stage_pack(store, &parsed, "user-import")?;
        let staged = staging::list_changes(store, Some(&batch_id), None)?.len();
        if apply.unwrap_or(false) {
            staging::set_review_status(store, &batch_id, None, "approved")?;
            apply::apply_batch(store, &batch_id, "user")?;
        }
        Ok(ImportOutcome { batch_id, staged })
    })
}

#[tauri::command]
fn validate_pack_file(state: State<AppState>, path: String) -> CmdResult<ValidateOutcome> {
    let json = std::fs::read_to_string(&path).map_err(|e| format!("تعذّرت قراءة الملف: {e}"))?;
    with_store(&state, |store| {
        let parsed = pack::parse_and_validate(&json, Some(store))?;
        Ok(ValidateOutcome { items: parsed.items.len(), producer: parsed.batch.producer })
    })
}

#[tauri::command]
fn source_manifest(state: State<AppState>) -> CmdResult<serde_json::Value> {
    with_store(&state, |store| {
        Ok(serde_json::json!({
            "sources": repo::list_sources(store)?,
            "tools": repo::list_tools(store, false)?,
            "tasks": repo::list_tasks(store, false)?,
            "itemTypes": pack::ITEM_TYPES
        }))
    })
}

// ---------------------------------------------------------------- backups, audit, diagnostics

#[tauri::command]
fn list_backups(state: State<AppState>) -> CmdResult<Vec<BackupRecord>> {
    with_store(&state, backup::list)
}

#[tauri::command]
fn create_backup(state: State<AppState>) -> CmdResult<BackupRecord> {
    with_store(&state, |store| backup::create(store, "manual", None))
}

#[tauri::command]
fn restore_backup(state: State<AppState>, id: String) -> CmdResult<BackupRecord> {
    with_store_mut(&state, |store| backup::restore(store, &id))
}

#[tauri::command]
fn list_audit(state: State<AppState>, limit: Option<i64>) -> CmdResult<Vec<AuditEntry>> {
    with_store(&state, |store| audit::list(store, limit.unwrap_or(40)))
}

#[tauri::command]
fn diagnostics(state: State<AppState>) -> CmdResult<Diagnostics> {
    with_store(&state, diagnostics::collect)
}

#[tauri::command]
fn rebuild_search_index(state: State<AppState>) -> CmdResult<i64> {
    with_store(&state, |store| search::rebuild(store.conn()))
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> CmdResult<serde_json::Map<String, serde_json::Value>> {
    with_store(&state, personal::all_settings)
}

#[tauri::command]
fn set_setting(state: State<AppState>, key: String, value: serde_json::Value) -> CmdResult<()> {
    with_store(&state, |store| personal::set_setting(store, &key, &value))
}

// ---------------------------------------------------------------- os integration

/// Copy text to the clipboard. Copying is all Personal Wiki ever does with a
/// command — it never executes one.
#[tauri::command]
fn copy_text(app: tauri::AppHandle, text: String) -> CmdResult<()> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(text).map_err(|e| e.to_string())
}

/// Open a URL in the user's browser after validating it in the core. Anything
/// that is not an absolute http(s) URL is refused here, not in the frontend.
#[tauri::command]
fn open_external(app: tauri::AppHandle, url: String) -> CmdResult<()> {
    use tauri_plugin_opener::OpenerExt;
    let safe = safe_url::validate_web_url(&url).map_err(to_message)?;
    app.opener().open_url(safe, None::<&str>).map_err(|e| e.to_string())
}

/// The only filesystem entry point: one picker, JSON only, chosen by the user.
#[tauri::command]
async fn pick_pack_file(app: tauri::AppHandle) -> CmdResult<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .add_filter("Knowledge Pack", &["json"])
        .pick_file(move |path| {
            let _ = tx.send(path.map(|p| p.to_string()));
        });
    rx.recv().map_err(|_| "تم إلغاء الاختيار".to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // One store for the whole process, migrated and seeded on first run.
            let db_path = paths::default_db_path();
            let mut store = Store::open(&db_path)?;
            seed::ensure_seeded(&mut store)?;
            app.manage(AppState { store: Mutex::new(store) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            home_summary, list_tools, tool_page, list_tasks, category_tree, category_path,
            task_detail, category_detail, items_for_tool, items_for_task, items_for_category,
            item_detail, record_open, list_tags, list_item_types, list_favorites, list_pinned,
            search, search_suggestions, search_history, clear_search_history, list_saved_searches,
            save_search, delete_saved_search, set_favorite, set_pin, list_notes, save_note,
            delete_note, list_links, save_link, delete_link, save_tool, save_task, save_category,
            save_item, set_active, list_sources, list_batches, list_changes, review_changes,
            apply_batch, revert_batch, import_pack_file, validate_pack_file, source_manifest,
            list_backups, create_backup, restore_backup, list_audit, diagnostics,
            rebuild_search_index, get_settings, set_setting, copy_text, open_external, pick_pack_file
        ])
        .run(tauri::generate_context!())
        .expect("Personal Wiki failed to start");
}
