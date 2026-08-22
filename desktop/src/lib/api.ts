/** Typed wrappers over the Tauri command surface. One function per command. */

import { call } from './ipc';
import type {
  ApplyOutcome, AuditEntry, BackupRecord, CategoryNode, Category, Diagnostics, HomeSummary,
  ItemDetail, ItemSummary, Note, SavedLink, SavedSearch, SearchFilters, SearchHistoryEntry,
  SearchResponse, Source, StagedChange, Tag, Task, TaskCard, Tool, ToolCard, ToolPage, UpdateBatch
} from './types';

// ---- browse
export const getHome = () => call<HomeSummary>('home_summary');
export const listTools = () => call<ToolCard[]>('list_tools');
export const getToolPage = (slug: string) => call<ToolPage>('tool_page', { slug });
export const listTasks = () => call<TaskCard[]>('list_tasks');
export const getCategoryTree = () => call<CategoryNode[]>('category_tree');
export const getCategoryPath = (slug: string) => call<Category[]>('category_path', { slug });
export const getTaskDetail = (slug: string) => call<Task>('task_detail', { slug });
export const getCategoryDetail = (slug: string) => call<Category>('category_detail', { slug });

export const itemsForTool = (slug: string, section: string | null, sort: string) =>
  call<ItemSummary[]>('items_for_tool', { slug, section, sort });
export const itemsForTask = (slug: string, sort: string) =>
  call<ItemSummary[]>('items_for_task', { slug, sort });
export const itemsForCategory = (slug: string, sort: string) =>
  call<ItemSummary[]>('items_for_category', { slug, sort });
export const getItem = (slug: string) => call<ItemDetail>('item_detail', { slug });
export const recordOpen = (itemId: string) => call<void>('record_open', { itemId });

export const listTags = () => call<Tag[]>('list_tags');
export const listItemTypes = () => call<string[]>('list_item_types');
export const listFavorites = () => call<ItemSummary[]>('list_favorites');
export const listPinned = () => call<ItemSummary[]>('list_pinned');

// ---- search
export const search = (query: string, filters: SearchFilters, limit = 25, offset = 0) =>
  call<SearchResponse>('search', { query, filters, limit, offset });
export const suggest = (query: string) => call<string[]>('search_suggestions', { query });
export const searchHistory = () => call<SearchHistoryEntry[]>('search_history');
export const clearSearchHistory = () => call<void>('clear_search_history');
export const listSavedSearches = () => call<SavedSearch[]>('list_saved_searches');
export const saveSearch = (name: string, query: string, filters: SearchFilters) =>
  call<SavedSearch>('save_search', { name, query, filters });
export const deleteSavedSearch = (id: string) => call<void>('delete_saved_search', { id });

// ---- personal layer
export const setFavorite = (itemId: string, favorite: boolean) =>
  call<boolean>('set_favorite', { itemId, favorite });
export const setPin = (itemId: string, pinned: boolean) => call<boolean>('set_pin', { itemId, pinned });
export const listNotes = () => call<Note[]>('list_notes');
export const saveNote = (input: { id?: string | null; itemId?: string | null; title: string; body: string }) =>
  call<Note>('save_note', { id: input.id ?? null, itemId: input.itemId ?? null, title: input.title, body: input.body });
export const deleteNote = (id: string) => call<void>('delete_note', { id });
export const listLinks = () => call<SavedLink[]>('list_links');
export const saveLink = (input: {
  id?: string | null; title: string; url: string; description?: string | null;
  toolId?: string | null; categoryId?: string | null; isFavorite?: boolean;
}) =>
  call<SavedLink>('save_link', {
    id: input.id ?? null, title: input.title, url: input.url,
    description: input.description ?? null, toolId: input.toolId ?? null,
    categoryId: input.categoryId ?? null, isFavorite: input.isFavorite ?? false
  });
export const deleteLink = (id: string) => call<void>('delete_link', { id });

// ---- editing (typed forms, validated in the core)
export const saveTool = (input: Record<string, unknown>) => call<Tool>('save_tool', { input });
export const saveTask = (input: Record<string, unknown>) => call<Task>('save_task', { input });
export const saveCategory = (input: Record<string, unknown>) => call<Category>('save_category', { input });
export const saveItem = (input: Record<string, unknown>) => call<ItemDetail>('save_item', { input });
export const setActive = (entity: string, id: string, active: boolean) =>
  call<void>('set_active', { entity, id, active });

// ---- updates
export const listSources = () => call<Source[]>('list_sources');
export const listBatches = () => call<UpdateBatch[]>('list_batches');
export const listChanges = (batchId: string | null, status: string | null) =>
  call<StagedChange[]>('list_changes', { batchId, status });
export const reviewChanges = (batchId: string, changeIds: string[] | null, status: string) =>
  call<number>('review_changes', { batchId, changeIds, status });
export const applyBatch = (batchId: string) => call<ApplyOutcome>('apply_batch', { batchId });
export const revertBatch = (batchId: string) => call<ApplyOutcome>('revert_batch', { batchId });
export const importPackFile = (path: string, apply: boolean) =>
  call<{ batchId: string; staged: number }>('import_pack_file', { path, apply });
export const validatePackFile = (path: string) => call<{ items: number; producer: string }>('validate_pack_file', { path });
export const sourceManifest = () => call<unknown>('source_manifest');

// ---- backups, audit, diagnostics, settings
export const listBackups = () => call<BackupRecord[]>('list_backups');
export const createBackup = () => call<BackupRecord>('create_backup');
export const restoreBackup = (id: string) => call<BackupRecord>('restore_backup', { id });
export const listAudit = (limit = 40) => call<AuditEntry[]>('list_audit', { limit });
export const getDiagnostics = () => call<Diagnostics>('diagnostics');
export const rebuildSearchIndex = () => call<number>('rebuild_search_index');
export const getSettings = () => call<Record<string, unknown>>('get_settings');
export const setSetting = (key: string, value: unknown) => call<void>('set_setting', { key, value });

// ---- os integration (least privilege: copy text, open validated https urls)
export const copyText = (text: string) => call<void>('copy_text', { text });
export const openExternal = (url: string) => call<void>('open_external', { url });
export const pickPackFile = () => call<string | null>('pick_pack_file');
