// Mirrors the serialized shapes of crates/pw-core/src/models.rs.
// Kept hand-written and small so the contract stays readable in both languages.

export type AccentKey = 'blue' | 'green' | 'purple' | 'orange' | 'teal' | 'red' | 'slate';
export type VerificationStatus = 'official' | 'verified' | 'community' | 'unverified' | 'demo';
export type LifecycleStatus = 'active' | 'experimental' | 'deprecated' | 'removed';

export interface Tool {
  id: string;
  slug: string;
  name: string;
  shortName: string | null;
  description: string | null;
  iconKey: string;
  accentKey: AccentKey;
  officialHomeUrl: string | null;
  sortOrder: number;
  isActive: boolean;
  externalKey: string | null;
  managedBy: string;
  updatedAt: string;
}

export interface ToolCard extends Tool {
  itemCount: number;
  exampleCount: number;
  noteCount: number;
  sourceCount: number;
  lastUpdatedAt: string | null;
}

export interface Task {
  id: string;
  slug: string;
  name: string;
  description: string | null;
  iconKey: string;
  sortOrder: number;
  isActive: boolean;
  externalKey: string | null;
}

export interface TaskCard extends Task {
  itemCount: number;
  toolCount: number;
  toolIcons: string[];
}

export interface Category {
  id: string;
  slug: string;
  name: string;
  description: string | null;
  iconKey: string;
  accentKey: AccentKey;
  parentId: string | null;
  sortOrder: number;
  isActive: boolean;
  externalKey: string | null;
}

export interface CategoryNode extends Category {
  itemCount: number;
  children: CategoryNode[];
}

export interface Tag {
  id: string;
  slug: string;
  name: string;
  accentKey: AccentKey;
}

export interface ItemSummary {
  id: string;
  slug: string;
  title: string;
  subtitle: string | null;
  shortDescription: string | null;
  itemType: string;
  syntax: string | null;
  verificationStatus: VerificationStatus;
  lifecycleStatus: LifecycleStatus;
  iconKey: string | null;
  accentKey: AccentKey | null;
  toolId: string | null;
  toolSlug: string | null;
  toolName: string | null;
  tags: Tag[];
  isFavorite: boolean;
  isPinned: boolean;
  hasExamples: boolean;
  hasSources: boolean;
  updatedAt: string;
}

export interface Example {
  id: string;
  title: string | null;
  description: string | null;
  code: string;
  expectedOutput: string | null;
  sortOrder: number;
}

export interface Parameter {
  id: string;
  name: string;
  description: string | null;
  isRequired: boolean;
  defaultValue: string | null;
  valueHint: string | null;
  sortOrder: number;
}

export interface ItemSourceRef {
  sourceId: string;
  name: string;
  sourceType: string;
  isOfficial: boolean;
  url: string | null;
  sourceUrl: string | null;
  sourceAnchor: string | null;
  sourceVersion: string | null;
  verifiedAt: string | null;
}

export interface RelatedItem {
  id: string;
  slug: string;
  title: string;
  relationshipType: string;
  toolSlug: string | null;
  iconKey: string | null;
  accentKey: AccentKey | null;
}

export interface DiagramNode {
  id: string;
  label: string;
  sublabel?: string | null;
  accentKey?: AccentKey | null;
  status?: 'ok' | 'pending' | 'warning' | null;
}

export interface DiagramEdge {
  from: string;
  to: string;
  label?: string | null;
}

export interface ResultDiagram {
  kind: 'flow' | 'fan_out';
  nodes: DiagramNode[];
  edges: DiagramEdge[];
}

export interface Note {
  id: string;
  itemId: string | null;
  itemTitle: string | null;
  title: string;
  body: string;
  createdAt: string;
  updatedAt: string;
}

export interface ItemDetail extends ItemSummary {
  fullDescription: string | null;
  whenToUse: string | null;
  expectedResult: string | null;
  warningText: string | null;
  resultDiagram: ResultDiagram | null;
  difficulty: string | null;
  lastVerifiedAt: string | null;
  managedBy: string;
  aliases: string[];
  examples: Example[];
  parameters: Parameter[];
  sources: ItemSourceRef[];
  related: RelatedItem[];
  tasks: Task[];
  categories: Category[];
  notes: Note[];
}

export interface SavedLink {
  id: string;
  title: string;
  url: string;
  description: string | null;
  toolId: string | null;
  toolName: string | null;
  categoryId: string | null;
  isFavorite: boolean;
  sortOrder: number;
  updatedAt: string;
}

export interface Source {
  id: string;
  toolId: string | null;
  toolName: string | null;
  name: string;
  sourceType: string;
  url: string | null;
  isOfficial: boolean;
  syncStrategy: string;
  isActive: boolean;
  checkIntervalMinutes: number;
  lastCheckedAt: string | null;
  lastSuccessAt: string | null;
  lastFailureAt: string | null;
  lastError: string | null;
  nextCheckAt: string | null;
  health: 'ok' | 'stale' | 'failing' | 'disabled' | 'unchecked';
  notes: string | null;
}

export interface SearchFilters {
  toolSlugs: string[];
  taskSlugs: string[];
  categorySlugs: string[];
  itemTypes: string[];
  tagSlugs: string[];
  verificationStatuses: string[];
  lifecycleStatuses: string[];
  favoritesOnly: boolean;
}

export const emptyFilters = (): SearchFilters => ({
  toolSlugs: [],
  taskSlugs: [],
  categorySlugs: [],
  itemTypes: [],
  tagSlugs: [],
  verificationStatuses: [],
  lifecycleStatuses: [],
  favoritesOnly: false
});

export interface SearchHit extends ItemSummary {
  score: number;
  relevance: number;
  matchedAlias: string | null;
}

export interface SearchResponse {
  query: string;
  normalizedQuery: string;
  total: number;
  tookMs: number;
  hits: SearchHit[];
}

export interface ActivityEntry {
  id: string;
  kind: string;
  title: string;
  detail: string | null;
  itemSlug: string | null;
  toolSlug: string | null;
  iconKey: string | null;
  accentKey: AccentKey | null;
  at: string;
}

export interface HomeSummary {
  tools: ToolCard[];
  tasks: TaskCard[];
  recentItems: ItemSummary[];
  recentActivity: ActivityEntry[];
  pendingChanges: number;
  totalItems: number;
  totalExamples: number;
  lastSyncAt: string | null;
  lastSyncStatus: string | null;
}

export interface ToolStats {
  itemCount: number;
  exampleCount: number;
  noteCount: number;
  sourceCount: number;
  lastUpdatedAt: string | null;
}

export interface ToolPage {
  tool: Tool;
  stats: ToolStats;
  sections: CategoryNode[];
  tags: Tag[];
}

export interface FieldDiff {
  field: string;
  before: string | null;
  after: string | null;
}

export interface StagedChange {
  id: string;
  batchId: string;
  entityType: string;
  entityKey: string;
  changeType: 'add' | 'update' | 'deprecate' | 'remove_candidate' | 'relink';
  title: string | null;
  detail: string | null;
  beforeJson: unknown;
  afterJson: unknown;
  fieldDiff: FieldDiff[];
  status: 'pending' | 'approved' | 'rejected' | 'applied' | 'failed';
  confidence: number | null;
  errorText: string | null;
  detectedAt: string;
}

export interface UpdateBatch {
  id: string;
  sourceId: string | null;
  createdBy: string;
  producer: string | null;
  summary: string | null;
  notes: string | null;
  status: string;
  backupId: string | null;
  createdAt: string;
  appliedAt: string | null;
  revertedAt: string | null;
  pendingCount: number;
  approvedCount: number;
  rejectedCount: number;
  appliedCount: number;
  addCount: number;
  updateCount: number;
  removeCount: number;
}

export interface ApplyOutcome {
  batchId: string;
  applied: number;
  failed: number;
  skipped: number;
  backupId: string | null;
  status: string;
}

export interface BackupRecord {
  id: string;
  path: string;
  reason: string;
  batchId: string | null;
  sizeBytes: number;
  schemaVersion: number;
  itemCount: number;
  createdAt: string;
}

export interface AuditEntry {
  id: string;
  at: string;
  actor: string;
  action: string;
  entityType: string | null;
  entityId: string | null;
  batchId: string | null;
  detail: unknown;
}

export interface SavedSearch {
  id: string;
  name: string;
  query: string;
  filters: SearchFilters;
  createdAt: string;
  lastRunAt: string | null;
}

export interface SearchHistoryEntry {
  id: string;
  query: string;
  resultCount: number;
  executedAt: string;
}

export interface Diagnostics {
  appVersion: string;
  schemaVersion: number;
  expectedSchemaVersion: number;
  dbPath: string;
  dbSizeBytes: number;
  integrity: string;
  foreignKeyViolations: number;
  ftsStatus: string;
  ftsRowCount: number;
  counts: Record<string, number>;
  lastBatch: UpdateBatch | null;
  lastBackup: BackupRecord | null;
  problems: string[];
}
