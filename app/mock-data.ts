import type { Dashboard, KnowledgeItem, Source } from "./types";

const sources: Source[] = [
  { id: "src-claude", name: "Claude Code Docs", host: "docs.anthropic.com", url: "https://docs.anthropic.com/", type: "وثائق رسمية", official: true, status: "connected" },
  { id: "src-codex", name: "Codex CLI Docs", host: "github.com/openai/codex", url: "https://github.com/openai/codex", type: "مستودع رسمي", official: true, status: "connected" },
  { id: "src-gemini", name: "Gemini CLI Docs", host: "cloud.google.com", url: "https://cloud.google.com/gemini", type: "وثائق رسمية", official: true, status: "connected" },
];

const item = (value: Omit<KnowledgeItem, "favorite" | "pinned">): KnowledgeItem => ({ ...value, favorite: false, pinned: false });

const items: KnowledgeItem[] = [
  item({ id: "item-claude-agents", slug: "claude-code-agents", title: "إنشاء وكيل فرعي لتحليل الكود", subtitle: "تشغيل وكيل متخصص داخل Claude Code", toolId: "tool-claude", toolName: "Claude Code", toolIcon: "sparkles", categoryName: "الوكلاء والوكلاء الفرعية", type: "slash_command", description: "استخدم وكيلًا فرعيًا عندما تحتاج إلى تقسيم مهمة كبيرة إلى تحليل مستقل ومحدد.", whenToUse: "عندما تريد مراجعة الكود أو البحث عن الأخطاء دون إيقاف المهمة الرئيسية.", syntax: "/agents ثم اكتب وصف المهمة", expected: "يعود الوكيل بنتيجة منفصلة يمكن مراجعتها قبل اعتمادها.", verification: "official", lifecycle: "active", tags: ["الوكلاء", "تحليل", "تقسيم المهام"], aliases: ["تشغيل وكيل فرعي", "تحليل قاعدة الكود"], examples: [{ id: "ex-claude", title: "مراجعة سريعة", description: "اطلب فحصًا مستقلًا للملفات.", code: "راجع الملفات الخاصة بتسجيل الدخول وابحث عن أخطاء التحقق.", expected: "قائمة بالأخطاء مع الملفات المتأثرة." }], source: sources[0] }),
  item({ id: "item-codex-agent", slug: "codex-agent-run", title: "تشغيل وكيل Codex لمهمة محددة", subtitle: "تفويض مهمة واضحة داخل المشروع", toolId: "tool-codex", toolName: "Codex CLI", toolIcon: "terminal", categoryName: "الوكلاء والوكلاء الفرعية", type: "cli_command", description: "شغّل وكيلًا بمهمة محددة مع الحفاظ على حدود واضحة للنتيجة المطلوبة.", whenToUse: "عندما تحتاج إلى تنفيذ مهمة منفصلة يمكن مراجعتها بعد الانتهاء.", syntax: "codex agent run <name> <task>", expected: "ينفذ الوكيل المهمة ويعرض سجل التنفيذ والنتائج.", verification: "official", lifecycle: "active", tags: ["الوكلاء", "تنفيذ"], aliases: ["تشغيل وكيل كودكس"], examples: [{ id: "ex-codex", title: "تحليل الاختبارات", description: "وكّل مهمة تحليل فشل الاختبارات.", code: "codex agent run test-auditor \"حلل أسباب فشل الاختبارات\"" }], source: sources[1] }),
  item({ id: "item-gemini-login", slug: "gemini-cli-login", title: "تسجيل الدخول إلى Gemini CLI", subtitle: "تجهيز الأداة لاستخدام حسابك", toolId: "tool-gemini", toolName: "Gemini CLI", toolIcon: "sparkles", categoryName: "الدخول والحسابات", type: "cli_command", description: "ابدأ جلسة موثقة قبل استخدام أوامر Gemini CLI داخل مشروعك.", whenToUse: "عند تشغيل الأداة لأول مرة أو بعد انتهاء صلاحية الجلسة.", syntax: "gemini auth login", expected: "يفتح المتصفح لإكمال تسجيل الدخول ثم يحفظ الجلسة محليًا.", verification: "official", lifecycle: "active", tags: ["دخول", "إعداد"], aliases: ["دخول جيميني"], examples: [], source: sources[2] }),
  item({ id: "item-git-rebase", slug: "git-rebase", title: "إعادة ترتيب تغييرات Git", subtitle: "تنظيف سجل التعديلات قبل المشاركة", toolId: "tool-git", toolName: "Git / GitHub", toolIcon: "git-branch", categoryName: "سير عمل Git", type: "cli_command", description: "اجمع تغييراتك في سجل واضح قبل إرسالها للفريق.", whenToUse: "قبل دمج فرع أو إرسال طلب مراجعة عندما يحتاج السجل إلى ترتيب.", syntax: "git rebase -i HEAD~3", expected: "يفتح محرر التغييرات لإعادة ترتيبها أو دمجها.", verification: "official", lifecycle: "active", tags: ["Git", "مراجعة"], aliases: ["تنظيف سجل جيت"], examples: [], source: sources[1] }),
  item({ id: "item-old-debug", slug: "old-debug-command", title: "أمر تشخيص قديم", subtitle: "مثال على معرفة انتهت صلاحيتها", toolId: "tool-claude", toolName: "Claude Code", toolIcon: "sparkles", categoryName: "اكتشاف الأخطاء", type: "command", description: "هذا السجل محفوظ للرجوع التاريخي فقط.", whenToUse: "لا تستخدمه في مشروع جديد.", syntax: "/old-debug", expected: "قد لا يعمل في الإصدارات الحالية.", verification: "unverified", lifecycle: "deprecated", tags: ["قديم"], aliases: ["تشخيص قديم"], examples: [], source: sources[0] }),
];

export const mockDashboard: Dashboard = {
  tools: [
    { id: "tool-claude", slug: "claude-code", name: "Claude Code", description: "أوامر عملية للممارسة اليومية", icon: "sparkles", accent: "orange", itemCount: 128, exampleCount: 42, updated: "منذ يوم" },
    { id: "tool-codex", slug: "codex-cli", name: "Codex CLI", description: "إدارة المهام والوكلاء بكفاءة", icon: "terminal", accent: "green", itemCount: 96, exampleCount: 31, updated: "منذ يومين" },
    { id: "tool-opencode", slug: "opencode", name: "OpenCode", description: "طرق التخصيص والعمل في الخلفية", icon: "braces", accent: "purple", itemCount: 74, exampleCount: 19, updated: "منذ 3 أيام" },
    { id: "tool-gemini", slug: "gemini-cli", name: "Gemini CLI", description: "أوامر الأدوات الذكية والمشاريع", icon: "sparkles", accent: "blue", itemCount: 88, exampleCount: 27, updated: "منذ يوم" },
    { id: "tool-git", slug: "git-github", name: "Git / GitHub", description: "سير العمل وإدارة التغييرات", icon: "git-branch", accent: "orange", itemCount: 110, exampleCount: 36, updated: "منذ يومين" },
    { id: "tool-vscode", slug: "vs-code", name: "VS Code", description: "اختصارات وإضافات تساعدك أسرع", icon: "code-2", accent: "blue", itemCount: 53, exampleCount: 22, updated: "منذ 4 أيام" },
    { id: "tool-openrouter", slug: "openrouter", name: "OpenRouter", description: "اختيار النماذج والبوابات الذكية", icon: "network", accent: "purple", itemCount: 62, exampleCount: 15, updated: "منذ 5 أيام" },
    { id: "tool-links", slug: "useful-links", name: "روابط مفيدة", description: "مواقع ومراجع للرجوع السريع", icon: "link", accent: "teal", itemCount: 41, exampleCount: 8, updated: "منذ 6 أيام" },
  ],
  tasks: [
    { id: "task-agents", slug: "agents", name: "الوكلاء والوكلاء الفرعية", description: "تقسيم المهام وتشغيل وكلاء متخصصين", icon: "workflow", accent: "purple", itemCount: 24, toolCount: 4 },
    { id: "task-auth", slug: "authentication", name: "الدخول والحسابات", description: "تسجيل الدخول وإدارة الجلسات", icon: "shield-check", accent: "green", itemCount: 12, toolCount: 4 },
    { id: "task-setup", slug: "project-setup", name: "إعداد المشروع", description: "تجهيز بيئة العمل والمجلدات", icon: "settings-2", accent: "blue", itemCount: 18, toolCount: 5 },
    { id: "task-debug", slug: "debugging", name: "اكتشاف الأخطاء", description: "الوصول للسبب وإصلاحه بسرعة", icon: "triangle-alert", accent: "orange", itemCount: 16, toolCount: 4 },
    { id: "task-git", slug: "git-workflow", name: "سير عمل Git", description: "إدارة الفروع والمراجعات", icon: "git-branch", accent: "orange", itemCount: 31, toolCount: 3 },
    { id: "task-automation", slug: "automation", name: "الأتمتة والعمل بالخلفية", description: "تشغيل الأعمال الطويلة بوضوح", icon: "refresh-cw", accent: "teal", itemCount: 15, toolCount: 3 },
    { id: "task-models", slug: "model-selection", name: "اختيار النماذج", description: "موازنة الجودة والسرعة والتكلفة", icon: "sparkles", accent: "purple", itemCount: 11, toolCount: 3 },
  ],
  items,
  sources,
  changes: [
    { id: "change-1", batchId: "batch-demo", type: "add", title: "تشغيل وكيل فرعي لتحليل قاعدة الكود", detail: "إضافة جديدة من Claude Code Docs", time: "اليوم، 10:20 ص", status: "pending", after: { title: "تشغيل وكيل فرعي لتحليل قاعدة الكود" } },
    { id: "change-2", batchId: "batch-demo", type: "update", title: "codex agent run", detail: "تحديث شرح الاستخدام وإضافة مثال", time: "اليوم، 10:19 ص", status: "pending", before: { description: "تشغيل وكيل" }, after: { description: "تشغيل وكيل لمهمة محددة" } },
    { id: "change-3", batchId: "batch-demo", type: "update", title: "Gemini CLI Documentation", detail: "تغيير رابط المصدر الرسمي", time: "اليوم، 10:18 ص", status: "pending", before: { url: "old-url" }, after: { url: "https://cloud.google.com/gemini" } },
    { id: "change-4", batchId: "batch-demo", type: "remove", title: "/old-debug", detail: "تمت إزالته من المصدر", time: "اليوم، 10:16 ص", status: "pending", before: { lifecycle: "active" }, after: { lifecycle: "deprecated" } },
  ],
  notes: [{ id: "note-1", title: "طريقة مراجعة الوكلاء", body: "أراجع النتيجة أولًا ثم أطلب تطبيق التعديل.", itemId: "item-claude-agents", updated: "اليوم، 09:40 ص" }],
  links: [{ id: "link-1", title: "Claude Code Documentation", url: "https://docs.anthropic.com/", description: "المصدر الرسمي لأوامر Claude Code", favorite: true, toolId: "tool-claude" }, { id: "link-2", title: "Codex CLI Repository", url: "https://github.com/openai/codex", description: "المستودع الرسمي لـ Codex CLI", favorite: false, toolId: "tool-codex" }],
  activity: [
    { id: "activity-1", title: "كشف أمر جديد: /explain", detail: "شرح الكود أو الخطأ بتفصيل مع أمثلة.", time: "اليوم، 10:20 ص", kind: "add" },
    { id: "activity-2", title: "تحديث: تشغيل وكيل فرعي", detail: "تم تحسين الوصف وإضافة مثال.", time: "اليوم، 10:19 ص", kind: "update" },
    { id: "activity-3", title: "تحديث رابط الوثائق", detail: "تم التحقق من مصدر Gemini CLI.", time: "اليوم، 10:18 ص", kind: "sync" },
  ],
};
