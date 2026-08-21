/**
 * UI chrome copy. Knowledge content comes from the database; only labels,
 * empty states and help text live here — which is what keeps a second language
 * a data change rather than a rewrite.
 */

export const ar = {
  app: { name: 'Personal Wiki', version: 'الإصدار' },

  nav: {
    home: 'لوحة التحكم',
    tools: 'الأدوات',
    tasks: 'الأوامر',
    links: 'الروابط',
    favorites: 'المفضلة',
    notes: 'الملاحظات',
    updates: 'التحديثات',
    settings: 'الإعدادات',
    diagnostics: 'التشخيص'
  },

  topbar: {
    searchPlaceholder: 'ماذا تريد أن تفعل؟',
    shortcut: 'Ctrl K',
    updates: 'تحديث',
    sync: 'مزامنة',
    settings: 'الإعدادات',
    minimize: 'تصغير',
    maximize: 'تكبير',
    close: 'إغلاق'
  },

  sync: {
    synced: 'متزامنة',
    running: 'جارٍ المزامنة',
    failed: 'تعذّرت المزامنة',
    never: 'لم تتم المزامنة بعد',
    last: 'آخر مزامنة'
  },

  home: {
    heroTitle: 'خريطتك الشخصية للأوامر والمعرفة',
    heroSubtitle:
      'يساعدك Personal Wiki في العثور على الأوامر والشروحات والأمثلة بسرعة وتنظيم معلوماتك التقنية في مكان واحد.',
    allSections: 'عرض جميع الأقسام',
    recentUpdates: 'آخر التحديثات',
    recentItems: 'العناصر المستخدمة مؤخرًا',
    quickActions: 'إجراءات سريعة',
    viewAll: 'عرض الكل',
    tasksTitle: 'ابدأ من المهمة',
    toolsTitle: 'الأدوات',
    quickUpdate: { title: 'مركز التحديثات', subtitle: 'عرض ما هو جديد' },
    quickImport: { title: 'استيراد الأوامر', subtitle: 'من ملف حزمة معرفة' },
    quickAddLink: { title: 'إضافة رابط جديد', subtitle: 'احفظ مرجعًا مفيدًا' }
  },

  item: {
    purpose: 'الهدف',
    whenToUse: 'متى يُستخدم',
    syntax: 'الصيغة',
    parameters: 'المعاملات (الخيارات)',
    example: 'مثال على الاستخدام',
    expectedResult: 'النتيجة المتوقعة',
    warnings: 'ملاحظات وتنبيهات',
    source: 'المصدر الرسمي',
    related: 'أوامر ذات صلة',
    myNotes: 'ملاحظاتي',
    lastUpdated: 'آخر تحديث',
    aliases: 'أسماء بديلة',
    copy: 'نسخ الأمر',
    copied: 'تم النسخ',
    favorite: 'إضافة إلى المفضلة',
    unfavorite: 'إزالة من المفضلة',
    pin: 'تثبيت',
    unpin: 'إلغاء التثبيت',
    openSource: 'فتح المصدر',
    backToList: 'الرجوع إلى القائمة',
    paramOption: 'الخيار',
    paramDescription: 'الوصف',
    paramRequired: 'مطلوب',
    paramDefault: 'الافتراضي',
    yes: 'نعم',
    no: 'لا',
    addNote: 'إضافة ملاحظة'
  },

  tool: {
    totalCommands: 'إجمالي الأوامر',
    examples: 'الأمثلة',
    notes: 'الملاحظات',
    sources: 'المصادر',
    lastUpdate: 'آخر تحديث',
    availableCommand: 'أمر متاح',
    practicalExample: 'مثال عملي',
    note: 'ملاحظة',
    trustedSource: 'مصدر موثوق',
    explainerTitle: 'كيف تنظّم هذه الصفحة؟',
    explainer: [
      { title: 'عنوان الصفحة والمسار', body: 'يوضح اسم الأداة الحالية وموقعك داخل الويكي.' },
      { title: 'تنقّل الفروع الفرعية', body: 'أقسام منطقية تمثل الفروع الداخلية للأوامر.' },
      { title: 'بطاقات الأوامر', body: 'كل بطاقة تمثل أمرًا واحدًا مع شرح مختصر وإجراءات سريعة.' },
      { title: 'لوحة التفاصيل', body: 'تساعدك على فهم بنية الصفحة وكيفية الاستفادة من المحتوى.' }
    ],
    helpTitle: 'هل تحتاج مساعدة؟',
    helpBody: 'استخدم البحث أو تصفّح الفروع للعثور على ما تحتاجه بسرعة.',
    sortRecent: 'الأحدث',
    sortAlpha: 'الأبجدي',
    sortPopular: 'الأكثر استخدامًا',
    sortLabel: 'ترتيب'
  },

  search: {
    title: 'البحث الذكي',
    filters: 'المرشحات',
    allTools: 'كل الأدوات',
    contentType: 'نوع المحتوى',
    tags: 'الوسوم',
    source: 'المصدر',
    favoritesOnly: 'المفضلة فقط',
    clearFilters: 'مسح المرشحات',
    resultsFound: (n: string, ms: string) => `تم العثور على ${n} نتيجة خلال ${ms} ثانية`,
    sortRelevance: 'الأكثر صلة',
    open: 'فتح',
    copy: 'نسخ',
    preview: 'معاينة',
    suggestions: 'اقتراحات بحث',
    recent: 'عمليات البحث الحديثة',
    saved: 'عمليات البحث المحفوظة',
    saveCurrent: 'حفظ البحث الحالي',
    clearAll: 'مسح الكل',
    relevance: 'صلة',
    more: (n: string) => `عرض المزيد من النتائج (${n})`,
    scopedPlaceholder: (tool: string) => `البحث في أوامر ${tool}…`,
    namePrompt: 'اسم البحث المحفوظ'
  },

  updates: {
    title: 'مركز التحديثات والمصادر',
    subtitle: 'نبقي قاعدة معرفتك محدثة باستيراد المحتوى من المصادر الرسمية وتتبع التغييرات قبل تطبيقها.',
    connectedSources: 'المصادر المتصلة',
    sourcesSubtitle: 'المصادر الرسمية التي يتم استيراد المحتوى منها.',
    addSource: 'إضافة مصدر جديد',
    manageSources: 'إدارة المصادر',
    lastSync: 'آخر مزامنة',
    nextSync: 'المزامنة التالية',
    pending: 'التحديثات المعلقة',
    overall: 'الحالة العامة',
    healthy: 'جميع المصادر سليمة',
    unhealthy: 'بعض المصادر تحتاج انتباهك',
    allChanges: 'كل التغييرات',
    added: 'إضافية',
    updated: 'محدثة',
    removed: 'محذوفة',
    timeline: 'خط زمني للمزامنة',
    fullLog: 'عرض السجل الكامل',
    approvalTitle: 'قائمة الموافقة (قبل التطبيق)',
    approvalSubtitle: 'يجب مراجعة التغييرات والموافقة عليها قبل إضافتها إلى قاعدة المعرفة.',
    review: 'مراجعة التغييرات',
    approveAll: 'الموافقة على الكل',
    rejectAll: 'رفض الكل',
    importFile: 'استيراد ملف',
    applyApproved: 'تطبيق التغييرات المعتمدة',
    undoLast: 'التراجع عن آخر دفعة',
    reversible: 'جميع التغييرات قابلة للتراجع. يتم الاحتفاظ بنسخة احتياطية قبل التطبيق.',
    totalItems: 'إجمالي العناصر',
    newCommands: 'أوامر جديدة',
    knowledgeStatus: 'حالة المعرفة',
    appStatus: 'تحديث التطبيق',
    appUpToDate: 'التطبيق محدّث',
    checkApp: 'التحقق من التحديثات',
    approve: 'موافقة',
    reject: 'رفض',
    before: 'قبل',
    after: 'بعد',
    batch: 'دفعة',
    applied: 'تم التطبيق',
    noPending: 'لا توجد تغييرات بانتظار المراجعة.'
  },

  common: {
    loading: 'جارٍ التحميل…',
    retry: 'إعادة المحاولة',
    cancel: 'إلغاء',
    save: 'حفظ',
    delete: 'حذف',
    edit: 'تعديل',
    add: 'إضافة',
    close: 'إغلاق',
    open: 'فتح',
    more: 'عرض المزيد',
    itemsCount: (n: string) => `${n} عنصرًا`,
    commandsCount: (n: string) => `${n} أمرًا`,
    availableCount: (n: string) => `${n} أمرًا متاحًا`,
    diagnostics: 'التشخيص',
    yes: 'نعم',
    no: 'لا'
  },

  empty: {
    tools: { title: 'لا توجد أدوات بعد', body: 'أضف أول أداة أو استورد حزمة معرفة.' },
    search: { title: 'لا توجد نتيجة', body: 'جرّب عبارة مهمة أو اسم أداة أو اسمًا بديلًا للأمر.' },
    favorites: { title: 'لا توجد مفضلة', body: 'اضغط النجمة في أي أمر لإضافته هنا.' },
    notes: { title: 'لا توجد ملاحظات', body: 'اكتب أول ملاحظة لتحفظ طريقتك في استخدام الأداة.' },
    links: { title: 'لا توجد روابط', body: 'احفظ أول رابط مفيد للرجوع إليه لاحقًا.' },
    items: { title: 'لا توجد عناصر هنا', body: 'اختر قسمًا آخر أو استورد حزمة معرفة جديدة.' },
    changes: { title: 'لا توجد تغييرات معلّقة', body: 'كل شيء محدّث. استورد حزمة معرفة لمراجعة تغييرات جديدة.' },
    activity: { title: 'لا يوجد نشاط بعد', body: 'ستظهر هنا التحديثات بعد أول استيراد.' }
  },

  errors: {
    generic: { title: 'حدث خطأ', body: 'تعذّر إكمال العملية.' },
    database: {
      title: 'مشكلة في قاعدة البيانات',
      body: 'تعذّرت قراءة قاعدة المعرفة المحلية. افتح التشخيص لمعرفة التفاصيل.'
    },
    source: { title: 'تعذّر فحص المصدر', body: 'المعرفة المحفوظة محليًا لا تزال متاحة.' },
    notFound: { title: 'الصفحة غير موجودة', body: 'الرابط الذي فتحته لم يعد صالحًا.' }
  },

  settings: {
    title: 'الإعدادات',
    general: 'عام',
    language: 'اللغة واتجاه الواجهة',
    startupScreen: 'الشاشة الافتتاحية',
    knowledge: 'المعرفة والمصادر',
    dataFolder: 'مجلد البيانات',
    backupNow: 'إنشاء نسخة احتياطية الآن',
    restore: 'استعادة نسخة',
    rebuildIndex: 'إعادة بناء فهرس البحث',
    appUpdates: 'تحديثات التطبيق',
    about: 'حول التطبيق',
    backups: 'النسخ الاحتياطية'
  },

  diagnostics: {
    title: 'التشخيص',
    subtitle: 'حالة قاعدة المعرفة المحلية.',
    appVersion: 'إصدار التطبيق',
    schema: 'إصدار المخطط',
    dbPath: 'مسار قاعدة البيانات',
    dbSize: 'حجم قاعدة البيانات',
    integrity: 'سلامة البيانات',
    foreignKeys: 'مفاتيح مرتبطة معطوبة',
    fts: 'فهرس البحث',
    counts: 'عدد السجلات',
    lastBatch: 'آخر دفعة تحديث',
    lastBackup: 'آخر نسخة احتياطية',
    problems: 'مشكلات',
    noProblems: 'لا توجد مشكلات.',
    copyReport: 'نسخ التقرير'
  },

  notes: { newNote: 'ملاحظة جديدة', titleField: 'العنوان', bodyField: 'النص', linkedItem: 'مرتبطة بـ', saved: 'تم الحفظ' },
  links: { newLink: 'رابط جديد', titleField: 'العنوان', urlField: 'الرابط', descField: 'الوصف', toolField: 'الأداة' }
} as const;

export type Strings = typeof ar;
export const t = ar;
