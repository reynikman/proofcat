const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const FEEDBACK_EMAIL = "proofcat.dit@gmail.com";
const logEvent = (message, level = "info") => { invoke("log_event", { level, message }).catch(() => {}); };

// ======================= i18n =======================
const I18N = {
  en: {
    addFile: "Inspect media…",
    settings: "Settings",
    language: "Language",
    feedbackLabel: "Feedback",
    logsLabel: "Logs",
    feedbackBtn: "Send feedback",
    logsBtn: "Open folder",
    feedbackIntro: "Describe the problem or idea here:",
    emptyTitle: "Choose a task",
    emptyInspectTitle: "Inspect media",
    emptyInspectDesc: "Read technical, camera and audio details. You can also choose a DCP folder to validate.",
    emptyOffloadTitle: "Copy with verification",
    emptyOffloadDesc: "Create copies on separate drives and verify every destination before the card is reused.",
    drop: "Drop media files or a DCP folder",
    tabSummary: "Overview", tabChecks: "Checks",
    fullView: "All fields",
    save: "Save report (.md)",
    copy: "Copy",
    copied: "Copied ✓",
    compare: "Compare",
    compareClose: "✕ Close compare",
    comparePick: "Pick two files to compare:",
    colField: "Field",
    analyzing: "Analyzing…",
    searchPlaceholder: "Search in output…",
    exifCompactEmpty: "No camera metadata outside container tags. Turn on All fields to inspect the complete ExifTool report.",
    savedPrefix: "Saved: ",
    noSummary: "MediaInfo returned no summary data — see the tool tabs.",
    empty: "(empty)",
    grpFile: "File", grpVideo: "Video", grpMedia: "Media", grpAudio: "Audio", grpAudioProduction: "Audio & production", grpImage: "Image", grpCamera: "Camera",
    lblName: "Name", lblSize: "Size", lblFormat: "Format", lblProfile: "Profile", lblDuration: "Duration",
    lblOverallBitrate: "Overall bitrate", lblRecorded: "Recorded", lblCodec: "Codec", lblResolution: "Resolution",
    lblAspect: "Aspect", lblFps: "Frame rate", lblBitrate: "Bitrate", lblBitrateMax: "Max bitrate", lblDepth: "Bit depth", lblColor: "Color",
    lblMonLut: "Monitoring LUT", lblResolveHint: "Resolve input",
    lblScan: "Scan", lblHdr: "HDR", lblRotation: "Rotation", lblChannels: "Channels", lblSampleRate: "Sample rate",
    lblCamera: "Camera", lblLens: "Lens", lblIso: "ISO", lblShutter: "Shutter", lblAperture: "Aperture",
    lblFocus: "Focal length", lblGps: "GPS",
    // camera / color / slate cards
    grpColor: "Color", grpSlate: "Slate / Sound", grpLoud: "Loudness (EBU R128)",
    lblFocal35: "Focal (35mm eq.)", lblEi: "EI / Exp. index", lblWb: "White balance",
    lblFocusDist: "Focus distance", lblShutterAngle: "Shutter angle", lblGain: "Gain",
    lblExpMode: "Exposure mode", lblSensor: "Sensor size",
    lblColorProfile: "Picture profile / Log", lblPrimaries: "Color primaries", lblTransfer: "Transfer (gamma)",
    lblMatrix: "Matrix", lblRange: "Color range", lblChroma: "Chroma", lblTimecode: "Timecode",
    lblScene: "Scene", lblTake: "Take", lblReel: "Reel / Tape", lblTracks: "Tracks", lblNote: "Note", lblRecorder: "Recorder",
    lblIntegrated: "Integrated", lblTruePeak: "True Peak", lblLra: "Range (LRA)",
    loudMeasure: "Measure loudness", loudMeasuring: "Measuring…", loudNoAudio: "No audio track",
    loudTargets: "Targets: YouTube −14 · Broadcast −23 LUFS · Peak ≤ −1 dBTP",
    // updates
    updatesLabel: "Updates", checkUpdates: "Check for updates", updChecking: "Checking…",
    updUpToDate: "You have the latest version", updAvailable: "Update available:",
    updInstall: "Download & install", updInstalling: "Downloading & installing…", updError: "Check failed",
    updPrompt: "ProofCat {version} is ready. Download and install it now?",
    // crash reporting (opt-in, default OFF)
    crashLabel: "Error reports",
    crashHint: "Off by default. When on, crash reports go only to your own server — no file names or paths. Applies after restart.",
    // delivery spec check (regular files)
    specLabel: "Delivery standard", specNone: "— choose a standard —",
    specVerdictTitle: "Delivery check", specChecks: "Checks",
    checksTitle: "Delivery check", checksProfileHint: "Use this only to validate a delivery requirement. It does not indicate that the file is damaged.", checksRun: "Check file", checksIdle: "Choose a delivery standard to run a check.", checksReady: "Ready to check:", checksTechnical: "Additional tools", checksTechnicalHint: "Run these only when the workflow or delivery standard requires them.",
    // custom profile editor
    specEditBtn: "Custom profile…", specEditTitle: "Custom delivery profile",
    specFName: "Name", specFNameHelp: "Shown in the delivery-standard menu", specFReq: "Required", specMissing: "Complete these fields",
    specFTarget: "Integrated loudness", specFTol: "Tolerance", specFTp: "Max true peak",
    specFTpHelp: "dBTP · optional", specFSr: "Sample rate", specFSrHelp: "Optional", specFFps: "Check frame rate and CFR",
    specFDelete: "Delete", specFSave: "Save",
    // batch table
    setTools: "Set tools", batchOpen: "Overview", batchClose: "✕ Close set overview", batchTitle: "Set overview",
    batchHint: "Highlighted = odd one out (should match across the set). Timecode / Reel differ per clip — shown for conform reference.",
    batchVerdict: "Check", batchNoData: "No comparable media files loaded.",
    // offload
    offloadOpen: "Copy with verification…", offloadClose: "← Back",
    offloadTitle: "Copy with verification",
    offSource: "Source", offPick: "Choose source…", offChange: "Change…",
    offDests: "Destinations", offAddDest: "Add destination…",
    offProfile: "Verification mode",
    amName: "Verified", amTag: "independent readback", amDesc: "Reads every destination back from disk. Required before a card can be marked safe to format.",
    fastName: "Fast", fastTag: "one-pass copy", fastDesc: "Makes a durable copy in one pass. It never marks a card safe to format.",
    recommended: "Default", offAdvanced: "Additional settings", offAdvancedHint: "Report hashes, contacts and completion behaviour",
    offExtraHashes: "Extra report checksums", offExtraHashesHint: "For workflows that specifically request these report hashes.",
    offMhlInfo: "ASC MHL evidence will be saved on every destination.", offMhlRequired: "Required for Safe to Format.",
    offStart: "Start offload", offCancel: "Cancel", offClear: "New offload",
    offPause: "⏸ Pause", offResume: "▶ Resume", offPausedTag: "paused",
    offSrcNote: "The source is never modified.",
    offPhaseScanning: "Scanning source…", offPhaseSourcePreRead: "Independently reading source…", offPhaseCheckingExisting: "Checking existing copies…", offPhaseCopying: "Copying", offPhaseDestinationVerify: "Reading destination back…", offPhaseRepairing: "Repairing failed replica…", offPhaseManualVerify: "Verifying MHL media…", offPhaseMhl: "Writing MHL…", offPhaseDone: "Done",
    offCopied: "Copied", offSkipped: "Skipped (already matched)", offFailed: "Failed",
    offBytes: "Data copied", offMhlOut: "MHL manifests", offShowMhl: "Show MHL", offErrList: "Errors",
    offCopyTitle: "COPY COMPLETE — destinations were not independently read back",
    offVerifiedTitle: "ARCHIVE VERIFIED — all selected destinations passed readback",
    offSafeTitle: "SAFE TO FORMAT — two independent destinations passed independent readback",
    offVerifyFolder: "Verify an existing archive…", offVerifyOk: "MHL verification passed", offVerifyFail: "MHL verification failed",
    offReadbackOk: "Readback verified", offReadbackFail: "Readback incomplete", offFileCopies: "verified file copies",
    offShowEvidence: "Show file results", offHideEvidence: "Hide file results",
    offReverify: "Re-verify all destinations", offReverifyHint: "Re-read every file and check it against the saved MHL manifests.",
    offVerifyPassed: "passed", offVerifyFailed: "failed", offVerifyMissing: "missing",
    offOverallProgress: "Overall progress", offCurrentFile: "File {current} of {total}", offTransferSpeed: "Transfer speed",
    offVerifyCancelled: "MHL verification cancelled", offVerifyCancelledByUser: "Verification cancelled by user.",
    offWarnSameVolume: "Destination {destination} is on the same physical volume as the source.", offWarnOnlyOneIndependent: "Only one independently verified destination exists; do not format the source media.", offWarnDestinationsSameVolume: "ArchiveMax destinations are on the same physical volume ({volume}); they do not count as independent backups.",
    offConditions: "Formatting conditions", offTechnicalDetails: "Technical details", offExport: "Save report", offWarnings: "Warnings",
    offContacts: "DIT contacts (optional)", offContactsHint: "One contact per line: Name | role | phone or email. Saved locally and included in this job's evidence.",
    offNotifyDone: "Notify when the offload finishes",
    offCherryPickWarning: "Choose the full original card or shoot folder. Individual-file selections are not supported; mixed loose selections require review before formatting.",
    offFailTitle: "OFFLOAD INCOMPLETE — some files failed. Do NOT format the card",
    offCancelledTitle: "OFFLOAD CANCELLED — copy incomplete. Do NOT format the card",
    // verdict hero — the decision the product exists for
    vSafeWord: "Safe to format", vSafeSub: "Two independent destinations passed independent readback.",
    vArchiveWord: "Job complete", vArchiveSub: "Archive verified against the source, but not confirmed on two independent destinations.",
    vCopyWord: "Copy complete", vCopySub: "Copied. Not independently verified.",
    vFailedWord: "Do not format", vFailedSub: "Copy or verification failed. The source card is still the only good copy.",
    vNotSafe: "Not safe to format",
    offReplicas: "verified replicas",
    offSafeActionTitle: "Two verified replicas exist",
    offSafeActionBody: "Formatting the source card is safe. Erase it in the camera or Disk Utility — ProofCat never touches the card itself.",
    offNotSafeNote: "Do not format. Keep the source card until two independent replicas are verified.",
    // mode
    modeSimple: "Simple", modeDit: "DIT",
    modeTitle: "Simple mode hides metadata, delivery and batch tools — leaving only copy, verify and the verdict.",
    // shell
    brandTag: "verifier", filesLabel: "Files",
    themeLabel: "Theme", themeLight: "Light", themeDark: "Dark",
    // checksum + frame scan
    grpChecksum: "SHA-256 for delivery", hashBtn: "Create SHA-256", hashBusy: "Hashing…", hashCopy: "Copy", hashHint: "Use when a client supplied a checksum or asks for one. It does not validate the file by itself.",
    grpScan: "Black / frozen frames", scanBtn: "Scan frames", scanBusy: "Scanning…",
    scanNone: "No black or frozen frames found", scanBlack: "Black", scanFreeze: "Frozen",
    scanHint: "Decodes the whole video — may take a while",
    // DCP
    dcpVerdict: "DCP validation", dcpNaming: "Naming (DCNC)", dcpStruct: "Package structure",
    dcpMedia: "Essence (from MXF)", dcpChecks: "Checks",
    dcpPass: "PASS — ready to deliver", dcpWarn: "WARNINGS — review first", dcpFail: "FAIL — will be rejected",
    dcpTitle: "Title", dcpType: "Content type", dcpAspect: "Aspect", dcpAudioF: "Audio", dcpResF: "Resolution",
    dcpStd: "Standard", dcpPkg: "Package", dcpDate: "Date", dcpReels: "Reels",
    // DaVinci input mapping
    grpDaVinci: "DaVinci Resolve", lblDvCS: "Input Color Space", lblDvGamma: "Input Gamma", lblDvBasis: "Detected from",
    dvBasisCap: "capture gamma (camera)", dvBasisTag: "file color tags",
  },
  ru: {
    addFile: "Проверить медиа…",
    settings: "Настройки",
    language: "Язык",
    feedbackLabel: "Обратная связь",
    logsLabel: "Логи",
    feedbackBtn: "Отправить фидбек",
    logsBtn: "Открыть папку",
    feedbackIntro: "Опиши проблему или идею здесь:",
    emptyTitle: "Выберите задачу",
    emptyInspectTitle: "Проверить медиа",
    emptyInspectDesc: "Посмотреть технические, камерные и звуковые данные. Можно выбрать папку DCP для проверки.",
    emptyOffloadTitle: "Копирование с проверкой",
    emptyOffloadDesc: "Создать копии на разных дисках и проверить каждое назначение до повторного использования карты.",
    drop: "Отпустите медиафайлы или папку DCP",
    tabSummary: "Обзор", tabChecks: "Проверки",
    fullView: "Все поля",
    save: "Сохранить отчёт (.md)",
    copy: "Копировать",
    copied: "Скопировано ✓",
    compare: "Сравнить",
    compareClose: "✕ Закрыть сравнение",
    comparePick: "Выбери два файла для сравнения:",
    colField: "Поле",
    analyzing: "Анализ…",
    searchPlaceholder: "Поиск по выводу…",
    exifCompactEmpty: "Нет камерных метаданных вне служебных тегов контейнера. Включите «Все поля», чтобы увидеть полный отчёт ExifTool.",
    savedPrefix: "Сохранено: ",
    noSummary: "MediaInfo не дал данных для сводки — смотри вкладки утилит.",
    empty: "(пусто)",
    grpFile: "Файл", grpVideo: "Видео", grpMedia: "Медиа", grpAudio: "Аудио", grpAudioProduction: "Звук и съёмочные данные", grpImage: "Изображение", grpCamera: "Камера",
    lblName: "Имя", lblSize: "Размер", lblFormat: "Формат", lblProfile: "Профиль", lblDuration: "Длительность",
    lblOverallBitrate: "Общий битрейт", lblRecorded: "Дата записи", lblCodec: "Кодек", lblResolution: "Разрешение",
    lblAspect: "Соотношение", lblFps: "Кадры/с", lblBitrate: "Битрейт", lblBitrateMax: "Макс. битрейт", lblDepth: "Глубина", lblColor: "Цвет",
    lblMonLut: "Мониторинг-LUT", lblResolveHint: "Ввод Resolve",
    lblScan: "Развёртка", lblHdr: "HDR", lblRotation: "Поворот", lblChannels: "Каналы", lblSampleRate: "Частота",
    lblCamera: "Камера", lblLens: "Объектив", lblIso: "ISO", lblShutter: "Выдержка", lblAperture: "Диафрагма",
    lblFocus: "Фокус", lblGps: "GPS",
    // camera / color / slate cards
    grpColor: "Цвет", grpSlate: "Слейт / Звук", grpLoud: "Громкость (EBU R128)",
    lblFocal35: "Фокус (экв. 35мм)", lblEi: "EI / Эксп. индекс", lblWb: "Баланс белого",
    lblFocusDist: "Дистанция фокуса", lblShutterAngle: "Угол затвора", lblGain: "Усиление",
    lblExpMode: "Режим экспозиции", lblSensor: "Сенсор",
    lblColorProfile: "Профиль / Log", lblPrimaries: "Первичные цвета", lblTransfer: "Передаточная (гамма)",
    lblMatrix: "Матрица", lblRange: "Диапазон", lblChroma: "Цветность", lblTimecode: "Таймкод",
    lblScene: "Сцена", lblTake: "Дубль", lblReel: "Кассета / Reel", lblTracks: "Дорожки", lblNote: "Заметка", lblRecorder: "Рекордер",
    lblIntegrated: "Интегральная", lblTruePeak: "True Peak", lblLra: "Диапазон (LRA)",
    loudMeasure: "Измерить громкость", loudMeasuring: "Измеряю…", loudNoAudio: "Нет аудиодорожки",
    loudTargets: "Эталон: YouTube −14 · вещание −23 LUFS · Peak ≤ −1 dBTP",
    // updates
    updatesLabel: "Обновления", checkUpdates: "Проверить обновления", updChecking: "Проверяю…",
    updUpToDate: "Установлена последняя версия", updAvailable: "Доступно обновление:",
    updInstall: "Скачать и установить", updInstalling: "Скачиваю и ставлю…", updError: "Проверка не удалась",
    updPrompt: "Готово обновление ProofCat {version}. Скачать и установить сейчас?",
    // crash reporting (opt-in, по умолчанию OFF)
    crashLabel: "Отчёты об ошибках",
    crashHint: "По умолчанию выкл. Если включить — отчёты о сбоях уходят только на ваш сервер, без имён и путей файлов. Применится после перезапуска.",
    // delivery spec check (regular files)
    specLabel: "Стандарт сдачи", specNone: "— выбрать стандарт —",
    specVerdictTitle: "Проверка сдачи", specChecks: "Проверки",
    checksTitle: "Проверка сдачи", checksProfileHint: "Нужна только для проверки требований к сдаче. Она не означает, что файл повреждён.", checksRun: "Проверить файл", checksIdle: "Выберите стандарт сдачи, чтобы запустить проверку.", checksReady: "Готово к проверке:", checksTechnical: "Дополнительные инструменты", checksTechnicalHint: "Запускайте их, только если этого требует задача или стандарт.",
    // custom profile editor
    specEditBtn: "Свой профиль…", specEditTitle: "Свой профиль сдачи",
    specFName: "Название", specFNameHelp: "Показывается в меню стандартов сдачи", specFReq: "обязательно", specMissing: "Заполните поля",
    specFTarget: "Интегральная громкость", specFTol: "Допуск", specFTp: "Макс. true peak",
    specFTpHelp: "dBTP · необязательно", specFSr: "Частота дискретизации", specFSrHelp: "Необязательно", specFFps: "Проверять частоту кадров и CFR",
    specFDelete: "Удалить", specFSave: "Сохранить",
    // batch table
    setTools: "Инструменты набора", batchOpen: "Сводка", batchClose: "✕ Закрыть сводку", batchTitle: "Сводка набора",
    batchHint: "Подсвечено = белая ворона (в пачке должно совпадать). Таймкод / Reel у клипов разные — показаны для сверки конформа.",
    batchVerdict: "Проверка", batchNoData: "Нет сравнимых медиафайлов.",
    // offload
    offloadOpen: "Копия с проверкой", offloadClose: "← Назад",
    offloadTitle: "Копирование с проверкой",
    offSource: "Источник", offPick: "Выбрать источник…", offChange: "Изменить…",
    offDests: "Назначения", offAddDest: "Добавить назначение…",
    offProfile: "Режим проверки",
    amName: "Проверенная копия", amTag: "независимая перечитка", amDesc: "Перечитывает каждое назначение с диска. Нужна, прежде чем карту можно будет форматировать.",
    fastName: "Быстрая копия", fastTag: "копирование в один проход", fastDesc: "Создаёт надёжную копию в один проход. Никогда не разрешает форматировать карту.",
    recommended: "По умолчанию", offAdvanced: "Дополнительные настройки", offAdvancedHint: "Хеши отчёта, контакты и уведомление",
    offExtraHashes: "Дополнительные хеши отчёта", offExtraHashesHint: "Только для процессов, где эти хеши явно требуются.",
    offMhlInfo: "Доказательство ASC MHL будет сохранено в каждом назначении.", offMhlRequired: "Нужно для статуса «можно форматировать».",
    offStart: "Начать копирование", offCancel: "Отменить", offClear: "Новое копирование",
    offPause: "⏸ Пауза", offResume: "▶ Продолжить", offPausedTag: "на паузе",
    offSrcNote: "Источник не модифицируется никогда.",
    offPhaseScanning: "Сканирую источник…", offPhaseSourcePreRead: "Независимо читаю источник…", offPhaseCheckingExisting: "Проверяю существующие копии…", offPhaseCopying: "Копирую", offPhaseDestinationVerify: "Перечитываю назначение…", offPhaseRepairing: "Восстанавливаю сбойную копию…", offPhaseManualVerify: "Проверяю файлы по MHL…", offPhaseMhl: "Пишу MHL…", offPhaseDone: "Готово",
    offCopied: "Скопировано", offSkipped: "Пропущено (уже совпадает)", offFailed: "Ошибки",
    offBytes: "Данных скопировано", offMhlOut: "MHL-манифесты", offShowMhl: "Показать MHL", offErrList: "Ошибки",
    offCopyTitle: "КОПИРОВАНИЕ ЗАВЕРШЕНО — независимой перечитки назначений не было",
    offVerifiedTitle: "АРХИВ СВЕРЕН — все выбранные назначения прошли перечитку",
    offSafeTitle: "КАРТУ МОЖНО ФОРМАТИРОВАТЬ — две независимые копии прошли независимую перечитку",
    offVerifyFolder: "Проверить существующий архив…", offVerifyOk: "Проверка MHL пройдена", offVerifyFail: "Проверка MHL не пройдена",
    offReadbackOk: "Перечитка пройдена", offReadbackFail: "Перечитка не пройдена", offFileCopies: "проверенных файловых копий",
    offShowEvidence: "Показать результаты по файлам", offHideEvidence: "Скрыть результаты по файлам",
    offReverify: "Повторно проверить все назначения", offReverifyHint: "Перечитать каждый файл и сверить его с сохранёнными MHL-манифестами.",
    offVerifyPassed: "пройдено", offVerifyFailed: "ошибок", offVerifyMissing: "отсутствует",
    offOverallProgress: "Общий прогресс", offCurrentFile: "Файл {current} из {total}", offTransferSpeed: "Скорость передачи",
    offVerifyCancelled: "Проверка MHL отменена", offVerifyCancelledByUser: "Проверка отменена пользователем.",
    offWarnSameVolume: "Назначение {destination} находится на том же физическом диске, что и источник.", offWarnOnlyOneIndependent: "Есть только одно независимо проверенное назначение; исходный носитель нельзя форматировать.", offWarnDestinationsSameVolume: "Назначения ArchiveMax находятся на одном физическом диске ({volume}); они не считаются независимыми копиями.",
    offConditions: "Условия для форматирования", offTechnicalDetails: "Технические детали", offExport: "Сохранить отчёт", offWarnings: "Предупреждения",
    offContacts: "Контакты DIT (необязательно)", offContactsHint: "По одному в строке: имя | роль | телефон или email. Сохраняются локально и входят в evidence этого job.",
    offNotifyDone: "Уведомить, когда копирование завершится",
    offCherryPickWarning: "Выбирай всю исходную карту или папку съёмки. Отдельные файлы не поддерживаются; смешанный выбор требует проверки до форматирования.",
    offFailTitle: "КОПИРОВАНИЕ НЕ ЗАВЕРШЕНО — есть ошибки. Карту НЕ форматировать",
    offCancelledTitle: "КОПИРОВАНИЕ ОТМЕНЕНО — копия неполная. Карту НЕ форматировать",
    // verdict hero — главное решение продукта
    vSafeWord: "Можно форматировать", vSafeSub: "Два независимых назначения прошли независимую перечитку.",
    vArchiveWord: "Работа выполнена", vArchiveSub: "Архив сверён с источником, но не подтверждён на двух независимых назначениях.",
    vCopyWord: "Копия создана", vCopySub: "Скопировано. Независимо не проверено.",
    vFailedWord: "Не форматировать", vFailedSub: "Копирование или проверка не удались. Исходная карта — по-прежнему единственная годная копия.",
    vNotSafe: "Форматировать нельзя",
    offReplicas: "проверенных копий",
    offSafeActionTitle: "Есть две проверенные копии",
    offSafeActionBody: "Форматировать исходную карту безопасно. Стирай её в камере или Дисковой утилите — ProofCat сам карту не трогает.",
    offNotSafeNote: "Не форматируй. Держи исходную карту, пока не проверены две независимые копии.",
    // режим
    modeSimple: "Простой", modeDit: "DIT",
    modeTitle: "Простой режим скрывает метаданные, спецификации и списки — остаётся только копирование, проверка и вердикт.",
    // оболочка
    brandTag: "верификатор", filesLabel: "Файлы",
    themeLabel: "Тема", themeLight: "Светлая", themeDark: "Тёмная",
    // checksum + frame scan
    grpChecksum: "SHA-256 для передачи", hashBtn: "Создать SHA-256", hashBusy: "Считаю…", hashCopy: "Копировать", hashHint: "Нужен, когда заказчик дал эталонную сумму или просит приложить checksum. Сам по себе файл не проверяет.",
    grpScan: "Чёрные / замёрзшие кадры", scanBtn: "Сканировать кадры", scanBusy: "Сканирую…",
    scanNone: "Чёрных и замёрзших кадров не найдено", scanBlack: "Чёрные", scanFreeze: "Стоп-кадр",
    scanHint: "Декодирует всё видео — может занять время",
    // DCP
    dcpVerdict: "Проверка DCP", dcpNaming: "Наименование (DCNC)", dcpStruct: "Структура пакета",
    dcpMedia: "Эссенция (из MXF)", dcpChecks: "Проверки",
    dcpPass: "ГОТОВ — можно отдавать", dcpWarn: "ЕСТЬ ЗАМЕЧАНИЯ — проверь", dcpFail: "НЕ ПРОЙДЁТ — отклонят",
    dcpTitle: "Название", dcpType: "Тип контента", dcpAspect: "Кадр", dcpAudioF: "Звук", dcpResF: "Разрешение",
    dcpStd: "Стандарт", dcpPkg: "Пакет", dcpDate: "Дата", dcpReels: "Рилы",
    // DaVinci input mapping
    grpDaVinci: "DaVinci Resolve", lblDvCS: "Input Color Space", lblDvGamma: "Input Gamma", lblDvBasis: "Определено по",
    dvBasisCap: "capture-гамме (камера)", dvBasisTag: "цвет-тегам файла",
  },
  zh: {
    addFile: "检查媒体…",
    settings: "设置",
    language: "语言",
    feedbackLabel: "反馈",
    logsLabel: "日志",
    feedbackBtn: "发送反馈",
    logsBtn: "打开文件夹",
    feedbackIntro: "在此描述问题或想法：",
    emptyTitle: "选择任务",
    emptyInspectTitle: "检查媒体",
    emptyInspectDesc: "查看技术、相机和音频信息；也可选择 DCP 文件夹进行验证。",
    emptyOffloadTitle: "拷贝存储卡",
    emptyOffloadDesc: "复制到不同驱动器，并在重复使用存储卡前验证每个目标。",
    drop: "放入媒体文件或 DCP 文件夹",
    tabSummary: "概览", tabChecks: "检查",
    fullView: "全部字段",
    save: "保存报告 (.md)",
    copy: "复制",
    copied: "已复制 ✓",
    compare: "对比",
    compareClose: "✕ 关闭对比",
    comparePick: "选择两个文件进行对比：",
    colField: "字段",
    analyzing: "分析中…",
    searchPlaceholder: "在输出中搜索…",
    exifCompactEmpty: "容器标签之外没有相机元数据。启用“全部字段”以查看完整的 ExifTool 报告。",
    savedPrefix: "已保存：",
    noSummary: "MediaInfo 未返回摘要数据 — 请查看工具标签页。",
    empty: "（空）",
    grpFile: "文件", grpVideo: "视频", grpMedia: "媒体", grpAudio: "音频", grpAudioProduction: "音频与拍摄数据", grpImage: "图像", grpCamera: "相机",
    lblName: "名称", lblSize: "大小", lblFormat: "格式", lblProfile: "配置", lblDuration: "时长",
    lblOverallBitrate: "总码率", lblRecorded: "拍摄时间", lblCodec: "编码格式", lblResolution: "分辨率",
    lblAspect: "画幅比例", lblFps: "帧率", lblBitrate: "码率", lblBitrateMax: "最大码率", lblDepth: "位深", lblColor: "色彩",
    lblMonLut: "监看 LUT", lblResolveHint: "Resolve 输入",
    lblScan: "扫描方式", lblHdr: "HDR", lblRotation: "旋转", lblChannels: "声道数", lblSampleRate: "采样率",
    lblCamera: "相机", lblLens: "镜头", lblIso: "ISO", lblShutter: "快门", lblAperture: "光圈",
    lblFocus: "焦距", lblGps: "GPS",
    // camera / color / slate cards
    grpColor: "色彩 / 编码", grpSlate: "场记 / 声音", grpLoud: "响度 (EBU R128)",
    lblFocal35: "等效焦距 (35mm)", lblEi: "EI / 曝光指数", lblWb: "白平衡",
    lblFocusDist: "对焦距离", lblShutterAngle: "快门角度", lblGain: "增益",
    lblExpMode: "曝光模式", lblSensor: "感光元件尺寸",
    lblColorProfile: "图像配置 / Log", lblPrimaries: "色彩基色", lblTransfer: "传递函数（伽马）",
    lblMatrix: "色彩矩阵", lblRange: "色彩范围", lblChroma: "色度采样", lblTimecode: "时间码",
    lblScene: "场次", lblTake: "条数", lblReel: "卷号 / 磁带", lblTracks: "音轨", lblNote: "备注", lblRecorder: "录音机",
    lblIntegrated: "综合响度", lblTruePeak: "真实峰值", lblLra: "响度范围 (LRA)",
    loudMeasure: "测量响度", loudMeasuring: "测量中…", loudNoAudio: "无音轨",
    loudTargets: "标准：YouTube −14 · 广播 −23 LUFS · 峰值 ≤ −1 dBTP",
    // updates
    updatesLabel: "更新", checkUpdates: "检查更新", updChecking: "检查中…",
    updUpToDate: "已是最新版本", updAvailable: "有可用更新：",
    updInstall: "下载并安装", updInstalling: "正在下载并安装…", updError: "检查失败",
    updPrompt: "ProofCat {version} 已可更新。现在下载并安装吗？",
    // crash reporting (opt-in, default OFF)
    crashLabel: "错误报告",
    crashHint: "默认关闭。开启后，崩溃报告只会发送到你自己的服务器 — 不含文件名或路径。重启后生效。",
    // delivery spec check (regular files)
    specLabel: "交付标准", specNone: "— 选择标准 —",
    specVerdictTitle: "交付检查", specChecks: "检查项",
    checksTitle: "交付检查", checksProfileHint: "仅在需要验证交付要求时使用；这并不表示文件已损坏。", checksRun: "检查文件", checksIdle: "选择交付标准后即可运行检查。", checksReady: "准备检查：", checksTechnical: "附加工具", checksTechnicalHint: "仅在工作流程或交付标准需要时运行。",
    // custom profile editor
    specEditBtn: "自定义配置…", specEditTitle: "自定义交付配置",
    specFName: "名称", specFNameHelp: "显示在交付标准菜单中", specFReq: "必填", specMissing: "请完成这些字段",
    specFTarget: "综合响度目标", specFTol: "容差", specFTp: "最大真实峰值",
    specFTpHelp: "dBTP · 可选", specFSr: "采样率", specFSrHelp: "可选", specFFps: "检查帧率和 CFR",
    specFDelete: "删除", specFSave: "保存",
    // batch table
    setTools: "素材组工具", batchOpen: "概览", batchClose: "✕ 关闭概览", batchTitle: "素材组概览",
    batchHint: "高亮 = 与组内其他项不一致（本应保持一致）。时间码 / 卷号每个片段不同 — 仅供套底参考。",
    batchVerdict: "检查", batchNoData: "没有可比较的媒体文件。",
    // offload
    offloadOpen: "拷卡…", offloadClose: "← 返回",
    offloadTitle: "拷贝",
    offSource: "源", offPick: "选择源…", offChange: "更改…",
    offDests: "目标", offAddDest: "添加目标…",
    offProfile: "验证模式",
    amName: "已验证", amTag: "独立回读", amDesc: "从磁盘重新读取每个目标。存储卡获得可格式化状态前必须使用此模式。",
    fastName: "快速", fastTag: "单次复制", fastDesc: "一次完成持久复制。它永远不会允许格式化存储卡。",
    recommended: "默认", offAdvanced: "附加设置", offAdvancedHint: "报告哈希、联系人和完成行为",
    offExtraHashes: "额外报告校验和", offExtraHashesHint: "仅用于明确要求这些校验和的工作流程。",
    offMhlInfo: "每个目标都会保存 ASC MHL 证据。", offMhlRequired: "获得“可格式化”状态的必要条件。",
    offStart: "开始拷贝", offCancel: "取消", offClear: "新建拷贝",
    offPause: "⏸ 暂停", offResume: "▶ 继续", offPausedTag: "已暂停",
    offSrcNote: "源文件永远不会被修改。",
    offPhaseScanning: "正在扫描源…", offPhaseSourcePreRead: "正在独立读取源文件…", offPhaseCheckingExisting: "正在检查已有副本…", offPhaseCopying: "正在复制", offPhaseDestinationVerify: "正在回读目标…", offPhaseRepairing: "正在修复失败的副本…", offPhaseManualVerify: "正在按 MHL 校验媒体…", offPhaseMhl: "正在写入 MHL…", offPhaseDone: "已完成",
    offCopied: "已复制", offSkipped: "已跳过（已匹配）", offFailed: "失败",
    offBytes: "已复制数据量", offMhlOut: "MHL 清单", offShowMhl: "显示 MHL", offErrList: "错误",
    offCopyTitle: "复制完成 — 尚未对目标进行独立回读校验",
    offVerifiedTitle: "归档已校验 — 所有选定目标均通过回读校验",
    offSafeTitle: "可以格式化 — 两个独立目标均通过独立回读",
    offVerifyFolder: "验证现有归档…", offVerifyOk: "MHL 校验通过", offVerifyFail: "MHL 校验未通过",
    offReadbackOk: "回读校验通过", offReadbackFail: "回读校验未完成", offFileCopies: "个已验证文件副本",
    offShowEvidence: "显示文件结果", offHideEvidence: "隐藏文件结果",
    offReverify: "重新验证所有目标", offReverifyHint: "重新读取每个文件，并与保存的 MHL 清单进行核对。",
    offVerifyPassed: "通过", offVerifyFailed: "失败", offVerifyMissing: "缺失",
    offOverallProgress: "总体进度", offCurrentFile: "文件 {current} / {total}", offTransferSpeed: "传输速度",
    offVerifyCancelled: "MHL 验证已取消", offVerifyCancelledByUser: "验证已由用户取消。",
    offWarnSameVolume: "目标 {destination} 与源位于同一物理磁盘上。", offWarnOnlyOneIndependent: "只有一个经过独立验证的目标；请勿格式化源介质。", offWarnDestinationsSameVolume: "ArchiveMax 目标位于同一物理磁盘上（{volume}）；它们不算独立备份。",
    offConditions: "格式化条件", offTechnicalDetails: "技术详情", offExport: "保存报告", offWarnings: "警告",
    offContacts: "DIT 联系人（可选）", offContactsHint: "每行一位：姓名 | 角色 | 电话或邮箱。保存在本机并写入此任务的证据。",
    offNotifyDone: "拷贝完成时通知我",
    offCherryPickWarning: "请选择完整的原始存储卡或拍摄文件夹。不支持选择单个文件；混合的零散选择需在格式化前人工复核。",
    offFailTitle: "拷贝未完成 — 部分文件失败，请勿格式化存储卡",
    offCancelledTitle: "拷贝已取消 — 复制未完成，请勿格式化存储卡",
    // verdict hero — the decision the product exists for
    vSafeWord: "可以格式化", vSafeSub: "两个独立目标均已通过独立回读。",
    vArchiveWord: "任务完成", vArchiveSub: "归档已与源文件核对，但尚未在两个独立目标上完成确认。",
    vCopyWord: "复制完成", vCopySub: "已复制，尚未独立校验。",
    vFailedWord: "请勿格式化", vFailedSub: "复制或校验失败。源存储卡仍是唯一可靠的副本。",
    vNotSafe: "不可格式化",
    offReplicas: "个已校验副本",
    offSafeActionTitle: "已存在两个已校验副本",
    offSafeActionBody: "可以安全地格式化源存储卡了。请在相机或磁盘工具中清除它 — ProofCat 本身从不接触存储卡。",
    offNotSafeNote: "请勿格式化。在两个独立副本通过校验之前，请保留源存储卡。",
    // mode
    modeSimple: "简易", modeDit: "DIT",
    modeTitle: "简易模式隐藏元数据、交付检查和批量工具 — 只保留复制、校验和最终结论。",
    // shell
    brandTag: "校验工具", filesLabel: "文件",
    themeLabel: "主题", themeLight: "浅色", themeDark: "深色",
    // checksum + frame scan
    grpChecksum: "交付用 SHA-256", hashBtn: "生成 SHA-256", hashBusy: "计算中…", hashCopy: "复制", hashHint: "仅在客户提供校验和或要求附上校验和时使用；它本身不验证文件。",
    grpScan: "黑场 / 静帧检测", scanBtn: "扫描帧", scanBusy: "扫描中…",
    scanNone: "未发现黑场或静帧", scanBlack: "黑场", scanFreeze: "静帧",
    scanHint: "需解码整段视频 — 可能耗时较长",
    // DCP
    dcpVerdict: "DCP 校验", dcpNaming: "命名规范 (DCNC)", dcpStruct: "包结构",
    dcpMedia: "本体（来自 MXF）", dcpChecks: "检查项",
    dcpPass: "通过 — 可以交付", dcpWarn: "有警告 — 请先检查", dcpFail: "未通过 — 将被拒收",
    dcpTitle: "标题", dcpType: "内容类型", dcpAspect: "画幅比例", dcpAudioF: "音频", dcpResF: "分辨率",
    dcpStd: "标准", dcpPkg: "包类型", dcpDate: "日期", dcpReels: "本数（Reel）",
    // DaVinci input mapping
    grpDaVinci: "DaVinci Resolve", lblDvCS: "Input Color Space", lblDvGamma: "Input Gamma", lblDvBasis: "判定依据",
    dvBasisCap: "拍摄伽马（相机）", dvBasisTag: "文件色彩标签",
  },
  ja: {
    addFile: "メディアを確認…",
    settings: "設定",
    language: "言語",
    feedbackLabel: "フィードバック",
    logsLabel: "ログ",
    feedbackBtn: "フィードバックを送る",
    logsBtn: "フォルダを開く",
    feedbackIntro: "問題やアイデアをここに記入してください：",
    emptyTitle: "タスクを選択",
    emptyInspectTitle: "メディアを確認",
    emptyInspectDesc: "技術・カメラ・音声情報を確認し、DCP フォルダも検証できます。",
    emptyOffloadTitle: "カードを取り込む",
    emptyOffloadDesc: "別のドライブへコピーし、カードを再利用する前に各コピー先を検証します。",
    drop: "メディアファイルまたは DCP フォルダをドロップ",
    tabSummary: "概要", tabChecks: "検証",
    fullView: "すべての項目",
    save: "レポートを保存 (.md)",
    copy: "コピー",
    copied: "コピーしました ✓",
    compare: "比較",
    compareClose: "✕ 比較を閉じる",
    comparePick: "比較する2つのファイルを選択：",
    colField: "項目",
    analyzing: "解析中…",
    searchPlaceholder: "出力内を検索…",
    exifCompactEmpty: "コンテナタグ以外にカメラメタデータはありません。すべての項目をオンにすると完全な ExifTool レポートを確認できます。",
    savedPrefix: "保存先：",
    noSummary: "MediaInfo からサマリーデータが返されませんでした — 各ツールのタブを確認してください。",
    empty: "（空）",
    grpFile: "ファイル", grpVideo: "映像", grpMedia: "メディア", grpAudio: "音声", grpAudioProduction: "音声・撮影データ", grpImage: "画像", grpCamera: "カメラ",
    lblName: "名前", lblSize: "サイズ", lblFormat: "フォーマット", lblProfile: "プロファイル", lblDuration: "尺",
    lblOverallBitrate: "総ビットレート", lblRecorded: "撮影日時", lblCodec: "コーデック", lblResolution: "解像度",
    lblAspect: "アスペクト比", lblFps: "フレームレート", lblBitrate: "ビットレート", lblBitrateMax: "最大ビットレート", lblDepth: "ビット深度", lblColor: "カラー",
    lblMonLut: "モニタリング LUT", lblResolveHint: "Resolve 入力",
    lblScan: "走査方式", lblHdr: "HDR", lblRotation: "回転", lblChannels: "チャンネル数", lblSampleRate: "サンプルレート",
    lblCamera: "カメラ", lblLens: "レンズ", lblIso: "ISO", lblShutter: "シャッター", lblAperture: "絞り",
    lblFocus: "焦点距離", lblGps: "GPS",
    // camera / color / slate cards
    grpColor: "カラー / コーデック", grpSlate: "スレート / 音声", grpLoud: "ラウドネス (EBU R128)",
    lblFocal35: "焦点距離（35mm 換算）", lblEi: "EI / 露光指数", lblWb: "ホワイトバランス",
    lblFocusDist: "フォーカス距離", lblShutterAngle: "シャッター角度", lblGain: "ゲイン",
    lblExpMode: "露出モード", lblSensor: "センサーサイズ",
    lblColorProfile: "ピクチャープロファイル / Log", lblPrimaries: "原色（色域）", lblTransfer: "伝達関数（ガンマ）",
    lblMatrix: "マトリクス", lblRange: "カラーレンジ", lblChroma: "クロマサンプリング", lblTimecode: "タイムコード",
    lblScene: "シーン", lblTake: "テイク", lblReel: "リール / テープ", lblTracks: "トラック", lblNote: "メモ", lblRecorder: "レコーダー",
    lblIntegrated: "統合ラウドネス", lblTruePeak: "トゥルーピーク", lblLra: "ラウドネスレンジ (LRA)",
    loudMeasure: "ラウドネスを測定", loudMeasuring: "測定中…", loudNoAudio: "音声トラックなし",
    loudTargets: "基準：YouTube −14 · 放送 −23 LUFS · ピーク ≤ −1 dBTP",
    // updates
    updatesLabel: "アップデート", checkUpdates: "アップデートを確認", updChecking: "確認中…",
    updUpToDate: "最新バージョンです", updAvailable: "利用可能なアップデート：",
    updInstall: "ダウンロードしてインストール", updInstalling: "ダウンロード・インストール中…", updError: "確認に失敗しました",
    updPrompt: "ProofCat {version} を利用できます。今すぐダウンロードしてインストールしますか？",
    // crash reporting (opt-in, default OFF)
    crashLabel: "エラーレポート",
    crashHint: "初期設定はオフです。オンにすると、クラッシュレポートはご自身のサーバーにのみ送信されます — ファイル名やパスは含みません。再起動後に有効になります。",
    // delivery spec check (regular files)
    specLabel: "納品規格", specNone: "— 規格を選択 —",
    specVerdictTitle: "納品チェック", specChecks: "チェック項目",
    checksTitle: "納品チェック", checksProfileHint: "納品要件を確認するときだけ使います。ファイルの破損を意味するものではありません。", checksRun: "ファイルをチェック", checksIdle: "納品規格を選択するとチェックを実行できます。", checksReady: "チェックの準備完了：", checksTechnical: "追加ツール", checksTechnicalHint: "ワークフローまたは納品規格で必要な場合のみ実行してください。",
    // custom profile editor
    specEditBtn: "カスタムプロファイル…", specEditTitle: "カスタム納品プロファイル",
    specFName: "名前", specFNameHelp: "納品規格メニューに表示されます", specFReq: "必須", specMissing: "入力してください",
    specFTarget: "統合ラウドネス目標値", specFTol: "許容誤差", specFTp: "最大トゥルーピーク",
    specFTpHelp: "dBTP · 任意", specFSr: "サンプルレート", specFSrHelp: "任意", specFFps: "フレームレートと CFR をチェック",
    specFDelete: "削除", specFSave: "保存",
    // batch table
    setTools: "セットツール", batchOpen: "概要", batchClose: "✕ 概要を閉じる", batchTitle: "セット概要",
    batchHint: "ハイライト = セット内で異なる項目（本来は揃うはず）。タイムコード / リールはクリップごとに異なるため — コンフォーム参照用に表示。",
    batchVerdict: "チェック", batchNoData: "比較できるメディアファイルがありません。",
    // offload
    offloadOpen: "カードを取り込む…", offloadClose: "← 戻る",
    offloadTitle: "取り込み",
    offSource: "コピー元", offPick: "コピー元を選択…", offChange: "変更…",
    offDests: "コピー先", offAddDest: "コピー先を追加…",
    offProfile: "検証モード",
    amName: "検証済み", amTag: "独立した読み戻し", amDesc: "各コピー先をディスクから読み直します。カードをフォーマット可能にするにはこのモードが必要です。",
    fastName: "高速", fastTag: "1 回のコピー", fastDesc: "耐久性のあるコピーを 1 回で作成します。カードのフォーマットを許可しません。",
    recommended: "既定", offAdvanced: "追加設定", offAdvancedHint: "レポートハッシュ、連絡先、完了時の動作",
    offExtraHashes: "追加レポートチェックサム", offExtraHashesHint: "これらのチェックサムを明示的に要求するワークフロー用です。",
    offMhlInfo: "ASC MHL の証跡は各コピー先に保存されます。", offMhlRequired: "フォーマット可能にするために必要です。",
    offStart: "取り込み開始", offCancel: "キャンセル", offClear: "新規取り込み",
    offPause: "⏸ 一時停止", offResume: "▶ 再開", offPausedTag: "一時停止中",
    offSrcNote: "コピー元が変更されることはありません。",
    offPhaseScanning: "コピー元をスキャン中…", offPhaseSourcePreRead: "コピー元を独立して読み込み中…", offPhaseCheckingExisting: "既存のコピーを確認中…", offPhaseCopying: "コピー中", offPhaseDestinationVerify: "コピー先を読み戻し中…", offPhaseRepairing: "失敗した複製を修復中…", offPhaseManualVerify: "MHL でメディアを検証中…", offPhaseMhl: "MHL を書き出し中…", offPhaseDone: "完了",
    offCopied: "コピー済み", offSkipped: "スキップ（既に一致）", offFailed: "失敗",
    offBytes: "コピー済みデータ量", offMhlOut: "MHL マニフェスト", offShowMhl: "MHL を表示", offErrList: "エラー",
    offCopyTitle: "コピー完了 — コピー先の独立読み戻し検証は未実施です",
    offVerifiedTitle: "アーカイブ検証済み — 選択したコピー先すべてが読み戻し検証に合格",
    offSafeTitle: "フォーマット可能 — 独立した2つのコピー先が読み戻し検証に合格",
    offVerifyFolder: "既存アーカイブを検証…", offVerifyOk: "MHL 検証に合格", offVerifyFail: "MHL 検証に失敗",
    offReadbackOk: "読み戻し検証済み", offReadbackFail: "読み戻し検証未完了", offFileCopies: "件の検証済みファイルコピー",
    offShowEvidence: "ファイル結果を表示", offHideEvidence: "ファイル結果を隠す",
    offReverify: "すべてのコピー先を再検証", offReverifyHint: "各ファイルを再読込し、保存済み MHL マニフェストと照合します。",
    offVerifyPassed: "合格", offVerifyFailed: "失敗", offVerifyMissing: "欠落",
    offOverallProgress: "全体の進捗", offCurrentFile: "ファイル {current} / {total}", offTransferSpeed: "転送速度",
    offVerifyCancelled: "MHL 検証をキャンセルしました", offVerifyCancelledByUser: "ユーザーが検証をキャンセルしました。",
    offWarnSameVolume: "コピー先 {destination} はコピー元と同じ物理ボリューム上にあります。", offWarnOnlyOneIndependent: "独立して検証済みのコピー先が 1 つしかありません。コピー元メディアをフォーマットしないでください。", offWarnDestinationsSameVolume: "ArchiveMax のコピー先は同じ物理ボリューム上にあります（{volume}）。独立したバックアップとしては扱われません。",
    offConditions: "フォーマットの条件", offTechnicalDetails: "技術詳細", offExport: "レポートを保存", offWarnings: "警告",
    offContacts: "DIT 連絡先（任意）", offContactsHint: "1行につき1件：名前 | 役割 | 電話番号またはメール。ローカルとこのジョブの証跡に保存されます。",
    offNotifyDone: "取り込み完了時に通知する",
    offCherryPickWarning: "元のカードまたは撮影フォルダを丸ごと選択してください。個別ファイルの選択には対応していません。混在した個別選択はフォーマット前に要確認です。",
    offFailTitle: "取り込み未完了 — 一部ファイルが失敗しました。カードをフォーマットしないでください",
    offCancelledTitle: "取り込みをキャンセルしました — コピーは未完了です。カードをフォーマットしないでください",
    // verdict hero — the decision the product exists for
    vSafeWord: "フォーマット可能", vSafeSub: "独立した2つのコピー先が読み戻し検証に合格しました。",
    vArchiveWord: "作業完了", vArchiveSub: "アーカイブはコピー元と照合済みですが、独立した2つのコピー先では未確認です。",
    vCopyWord: "コピー完了", vCopySub: "コピーは完了しましたが、独立検証はまだです。",
    vFailedWord: "フォーマットしないでください", vFailedSub: "コピーまたは検証に失敗しました。元のカードが依然として唯一の有効な複製です。",
    vNotSafe: "フォーマット不可",
    offReplicas: "件の検証済み複製",
    offSafeActionTitle: "検証済みの複製が2件あります",
    offSafeActionBody: "元のカードをフォーマットしても安全です。カメラまたはディスクユーティリティで消去してください — ProofCat がカード自体に触れることはありません。",
    offNotSafeNote: "フォーマットしないでください。独立した2つの複製が検証されるまで、元のカードを保管してください。",
    // mode
    modeSimple: "シンプル", modeDit: "DIT",
    modeTitle: "シンプルモードではメタデータ・納品チェック・バッチ機能を非表示にし、コピー・検証・判定のみを残します。",
    // shell
    brandTag: "検証ツール", filesLabel: "ファイル",
    themeLabel: "テーマ", themeLight: "ライト", themeDark: "ダーク",
    // checksum + frame scan
    grpChecksum: "納品用 SHA-256", hashBtn: "SHA-256 を作成", hashBusy: "計算中…", hashCopy: "コピー", hashHint: "依頼元からチェックサムが渡された場合、または添付を求められた場合に使います。単体ではファイルを検証しません。",
    grpScan: "黒み / フリーズフレーム検出", scanBtn: "フレームをスキャン", scanBusy: "スキャン中…",
    scanNone: "黒みフレーム・フリーズフレームは見つかりませんでした", scanBlack: "黒み", scanFreeze: "フリーズ",
    scanHint: "映像全体をデコードするため時間がかかる場合があります",
    // DCP
    dcpVerdict: "DCP 検証", dcpNaming: "命名規則 (DCNC)", dcpStruct: "パッケージ構造",
    dcpMedia: "エッセンス（MXF より）", dcpChecks: "チェック項目",
    dcpPass: "合格 — 納品可能", dcpWarn: "警告あり — 先に確認してください", dcpFail: "不合格 — 受理されません",
    dcpTitle: "タイトル", dcpType: "コンテンツ種別", dcpAspect: "アスペクト比", dcpAudioF: "音声", dcpResF: "解像度",
    dcpStd: "規格", dcpPkg: "パッケージ", dcpDate: "日付", dcpReels: "リール数",
    // DaVinci input mapping
    grpDaVinci: "DaVinci Resolve", lblDvCS: "Input Color Space", lblDvGamma: "Input Gamma", lblDvBasis: "判定根拠",
    dvBasisCap: "撮影ガンマ（カメラ）", dvBasisTag: "ファイルのカラータグ",
  },
};
let lang = localStorage.getItem("lang") || "en";
let theme = localStorage.getItem("theme") === "light" ? "light" : "dark";
let mode = localStorage.getItem("mode") === "simple" ? "simple" : "dit";

function applyTheme() {
  document.documentElement.dataset.theme = theme;
  document.querySelectorAll("[data-theme-set]").forEach((b) => {
    b.classList.toggle("active", b.dataset.themeSet === theme);
  });
}
const t = (key, lng = lang) => (I18N[lng] && I18N[lng][key]) || I18N.en[key] || key;

// The core deliberately keeps evidence messages stable and in English. Convert
// the few operator-facing, structured messages here, while preserving paths.
function localizeOffloadMessage(message) {
  const raw = String(message || "").trim();
  if (!raw) return "";
  if (/^Verification cancelled by user\.?$/i.test(raw)) return t("offVerifyCancelledByUser");

  const sourceVolume = raw.match(/^Destination\s+(.+?)\s+is on the same physical volume as the source\.?$/i);
  if (sourceVolume) return t("offWarnSameVolume").replace("{destination}", sourceVolume[1]);

  if (/^Only one independently verified destination exists; source media must not be formatted\.?$/i.test(raw)) {
    return t("offWarnOnlyOneIndependent");
  }

  const duplicateDestinations = raw.match(/^ArchiveMax destinations are on the same physical volume \((.+)\); they do not count as independent backups\.?$/i);
  if (duplicateDestinations) return t("offWarnDestinationsSameVolume").replace("{volume}", duplicateDestinations[1]);

  return raw;
}

function isCancelledVerification(message) {
  return /^Verification cancelled by user\.?$/i.test(String(message || "").trim());
}

// ======================= state =======================
const files = [];
let current = -1;
let seq = 0;
let currentTab = "summary";
let rawFull = false;
let searchTerm = "";
let matchIdx = 0;
let compareMode = false;
let cmpA = null, cmpB = null;
let batchMode = false;
let offloadMode = false;

// Welcome intentionally starts compact. Workspaces request their own minimum
// size, and the window only grows — it never shrinks a user-arranged view.
function ensureWorkspaceWindow(workspace = "inspect") {
  invoke("ensure_workspace_window", { workspace }).catch((error) => {
    logEvent(`workspace window resize unavailable: ${String(error)}`, "warn");
  });
}
// Состояние копирования живёт отдельно от files — оно не связано с инспекцией.
function readOffloadContacts() {
  try {
    const value = JSON.parse(localStorage.getItem("offloadContacts") || "[]");
    return Array.isArray(value) ? value : [];
  } catch (_) { return []; }
}
function contactsToText(contacts) {
  return contacts.map((contact) => [contact.name, contact.role, contact.contact].map((v) => String(v || "").trim()).join(" | ")).join("\n");
}
function parseContacts(text) {
  return text.split("\n").map((line) => {
    const [name = "", role = "", contact = ""] = line.split("|", 3).map((part) => part.trim());
    return { name, role, contact };
  }).filter((contact) => contact.name);
}
const off = {
  source: "", dests: [], profile: "archiveMax", extras: [],
  contacts: readOffloadContacts(),
  notifyWhenDone: localStorage.getItem("offloadNotifyWhenDone") !== "false",
  advancedOpen: false,
  running: false, paused: false, prog: null, summary: null, verifyReport: null, verifyError: "", evidenceExpanded: false, jobId: "", error: "",
  transferSpeedBps: 0, transferSpeedSample: null,
};
// Сброс в чистую передачу: карты выбираются заново. Настройки отчёта сохраняем.
function resetOffloadSpeed() { off.transferSpeedBps = 0; off.transferSpeedSample = null; }
function offResetFresh() { off.source = ""; off.dests = []; off.prog = null; off.summary = null; off.verifyReport = null; off.verifyError = ""; off.evidenceExpanded = false; off.error = ""; off.paused = false; resetOffloadSpeed(); }
// The prior UI opened on Netflix. Adopt the common online-delivery guide once, while preserving
// any other explicit choice (broadcast or a custom profile).
const SPEC_PROFILE_DEFAULT_VERSION = "youtube-social-v1";
const storedSpecProfile = localStorage.getItem("specProfile");
const shouldAdoptSocialDefault = localStorage.getItem("specProfileDefaultVersion") !== SPEC_PROFILE_DEFAULT_VERSION;
let specProfile = shouldAdoptSocialDefault && [null, "none", "netflix"].includes(storedSpecProfile)
  ? "social"
  : (storedSpecProfile || "social");
localStorage.setItem("specProfileDefaultVersion", SPEC_PROFILE_DEFAULT_VERSION);

// ======================= helpers =======================
const num = (v) => {
  const n = parseFloat(String(v ?? "").replace(",", "."));
  return Number.isFinite(n) ? n : null;
};
const pathBasename = (p) => p.split(/[\\/]/).filter(Boolean).pop() || p;
function fmtSize(bytes) {
  if (bytes == null) return "—";
  const u = ["B", "KB", "MB", "GB", "TB"];
  let n = Number(bytes), i = 0;
  while (n >= 1024 && i < u.length - 1) { n /= 1024; i++; }
  return `${i === 0 ? n : n.toFixed(2)} ${u[i]}`;
}
function fmtTransferSpeed(bytesPerSecond) {
  return `${fmtSize(Math.max(0, bytesPerSecond || 0))}/s`;
}
function template(key, values) {
  return Object.entries(values).reduce((text, [name, value]) => text.replace(`{${name}}`, String(value)), t(key));
}
function offloadTaskProgress(progress) {
  const total = Number(progress.totalFiles) || 0;
  const index = Math.max(1, Number(progress.fileIndex) || 1);
  if (!total) return { percent: 0, current: index, total };
  if (["mhl", "done"].includes(progress.phase)) return { percent: 100, current: total, total };

  // "copying" reports already-processed bytes for skipped existing files.
  // During an active file copy, calculate the whole-job progress from the file
  // index so the bar never restarts at zero for the next file.
  let fraction;
  if (progress.phase === "copying" && Number(progress.bytesTotal) > 0) {
    fraction = Number(progress.bytesDone || 0) / Number(progress.bytesTotal);
  } else {
    const withinFile = ["copyingData", "repairingData"].includes(progress.phase) && Number(progress.bytesTotal) > 0
      ? Number(progress.bytesDone || 0) / Number(progress.bytesTotal)
      : 0;
    fraction = (index - 1 + withinFile) / total;
  }
  return { percent: Math.round(Math.min(1, Math.max(0, fraction)) * 100), current: Math.min(index, total), total };
}
function updateOffloadTransferSpeed(progress) {
  if (!["copyingData", "repairingData"].includes(progress.phase)) {
    resetOffloadSpeed();
    return;
  }
  const bytes = Number(progress.bytesDone);
  if (!Number.isFinite(bytes)) return;
  const now = performance.now();
  const fileKey = `${progress.phase}:${progress.currentFile || ""}`;
  const previous = off.transferSpeedSample;
  if (!previous || previous.fileKey !== fileKey || bytes < previous.bytes) {
    off.transferSpeedSample = { fileKey, bytes, at: now };
    return;
  }
  const elapsed = (now - previous.at) / 1000;
  if (elapsed >= 0.15) {
    const instant = (bytes - previous.bytes) / elapsed;
    if (instant >= 0) {
      off.transferSpeedBps = off.transferSpeedBps
        ? off.transferSpeedBps * 0.35 + instant * 0.65
        : instant;
    }
    off.transferSpeedSample = { fileKey, bytes, at: now };
  }
}
function fmtDur(sec) {
  const s = num(sec);
  if (s == null) return "—";
  const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), ss = s % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, "0")}:${String(Math.floor(ss)).padStart(2, "0")}`;
  return `${m}:${ss.toFixed(1).padStart(4, "0")}`;
}
function fmtBitrate(bps) {
  const n = num(bps);
  if (n == null) return null;
  return n >= 1e6 ? `${(n / 1e6).toFixed(2)} Mb/s` : `${(n / 1e3).toFixed(0)} kb/s`;
}
function fmtRate(hz) {
  const n = num(hz);
  return n == null ? null : `${(n / 1000).toFixed(1)} kHz`;
}
function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
}
const escRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const L = (ru, en) => (lang === "ru" ? ru : en); // локализованная строка для динамических сообщений DCP

// ======================= parse =======================
function miTracks(report) {
  try {
    const j = JSON.parse(report.summary_mi_json);
    let tr = j?.media?.track;
    return tr ? (Array.isArray(tr) ? tr : [tr]) : [];
  } catch { return []; }
}
const trackOf = (tracks, type) => tracks.find((x) => String(x["@type"]).toLowerCase() === type) || null;
function exifObj(report) {
  try {
    const j = JSON.parse(report.summary_exif_json);
    return Array.isArray(j) ? j[0] : j;
  } catch { return null; }
}
function exifPick(obj, tail) {
  if (!obj) return null;
  const k = Object.keys(obj).find((key) => key.split(":").pop() === tail);
  return k ? obj[k] : null;
}
function exifPickFirst(obj, tails) {
  for (const tail of tails) {
    const v = exifPick(obj, tail);
    if (v != null && v !== "") return v;
  }
  return null;
}
// Sony rtmd / прочие поля живут в mediainfo Other-треке под `extra`; берём _String, если есть.
function rtmd(oth, base) {
  const e = oth && oth.extra;
  if (!e) return null;
  const v = e[base + "_String"] ?? e[base];
  return v == null || v === "" ? null : v;
}

// Sony XAVC пишет истинную гамму/гамут съёмки в блок AcquisitionRecord (CameraUnitMetadataSet),
// а НЕ в теги контейнера (там всегда BT.709 — ловушка). В exiftool JSON пары name/value
// схлопываются (выживает одна), поэтому парсим плоский текст вкладки ExifTool, где обе на месте.
function sonyAcq(f) {
  const txt = f && f.report && f.report.exiftool && f.report.exiftool.output;
  if (!txt) return {};
  const map = {};
  let pendingName = null;
  for (const ln of txt.split(/\r?\n/)) {
    const mn = ln.match(/Acquisition\s*Record\s*Group\s*Item\s*Name\s*:\s*(.+?)\s*$/i);
    if (mn) { pendingName = mn[1].trim(); continue; }
    const mv = ln.match(/Acquisition\s*Record\s*Group\s*Item\s*Value\s*:\s*(.+?)\s*$/i);
    if (mv && pendingName) { map[pendingName] = mv[1].trim(); pendingName = null; }
  }
  return map;
}

// Универсальный разбор кривой (лога) из произвольной строки метаданных любого бренда.
// Возвращает {gamma, vendor, dvGamma} или null. Порядок важен: сначала специфичные лог-профили.
function logFromStr(s) {
  const g = String(s || "").toLowerCase();
  if (!g) return null;
  if (/s-?log3/.test(g)) return { gamma: "S-Log3", vendor: "Sony", dvGamma: "Sony S-Log3" };
  if (/s-?log2/.test(g)) return { gamma: "S-Log2", vendor: "Sony", dvGamma: "Sony S-Log2" };
  if (/\bs-?log\b/.test(g)) return { gamma: "S-Log", vendor: "Sony", dvGamma: "Sony S-Log" };
  if (/v-?log/.test(g)) return { gamma: "V-Log", vendor: "Panasonic", dvGamma: "Panasonic V-Log" };
  if (/log-?c4|logc4/.test(g)) return { gamma: "LogC4", vendor: "ARRI", dvGamma: "ARRI LogC4" };
  if (/log-?c3|logc3|log-?c|logc/.test(g)) return { gamma: "LogC3", vendor: "ARRI", dvGamma: "ARRI LogC3 (EI800)" };
  if (/canon ?log ?3|c-?log ?3|clog3/.test(g)) return { gamma: "Canon Log 3", vendor: "Canon", dvGamma: "Canon Log 3" };
  if (/canon ?log ?2|c-?log ?2|clog2/.test(g)) return { gamma: "Canon Log 2", vendor: "Canon", dvGamma: "Canon Log 2" };
  if (/canon ?log|c-?log|clog/.test(g)) return { gamma: "Canon Log", vendor: "Canon", dvGamma: "Canon Log" };
  if (/f-?log ?2|flog2/.test(g)) return { gamma: "F-Log2", vendor: "Fujifilm", dvGamma: "Fujifilm F-Log2" };
  if (/f-?log|flog/.test(g)) return { gamma: "F-Log", vendor: "Fujifilm", dvGamma: "Fujifilm F-Log" };
  if (/n-?log|nlog/.test(g)) return { gamma: "N-Log", vendor: "Nikon", dvGamma: "Nikon N-Log" };
  if (/d-?log-?m|dlogm/.test(g)) return { gamma: "D-Log M", vendor: "DJI", dvGamma: "DJI D-Log" };
  if (/d-?log|dlog/.test(g)) return { gamma: "D-Log", vendor: "DJI", dvGamma: "DJI D-Log" };
  if (/log3g10/.test(g)) return { gamma: "Log3G10", vendor: "RED", dvGamma: "REDLog3G10" };
  if (/redlogfilm|red ?log/.test(g)) return { gamma: "REDLogFilm", vendor: "RED", dvGamma: "REDLogFilm" };
  if (/gen ?5|blackmagic|bmd ?film|braw/.test(g)) return { gamma: "Blackmagic Film", vendor: "Blackmagic", dvGamma: "Blackmagic Design" };
  if (/2100 ?hlg|\bhlg\b|arib ?b67/.test(g)) return { gamma: "HLG", vendor: null, dvGamma: "Rec.2100 HLG" };
  if (/2084|st ?2084|\bpq\b|perceptual quant/.test(g)) return { gamma: "PQ", vendor: null, dvGamma: "ST2084" };
  if (/bt\.?709|rec\.?709|\b709\b|gamma ?2\.4|iec ?61966/.test(g)) return { gamma: "Rec.709", vendor: null, dvGamma: "Gamma 2.4" };
  if (/bt\.?2020|rec\.?2020/.test(g)) return { gamma: "Rec.2020", vendor: null, dvGamma: "Gamma 2.4" };
  return null;
}
// Разбор гамута (цветового охвата) из строки. Возвращает {gamut, dvCS} или null.
function gamutFromStr(s) {
  const g = String(s || "").toLowerCase();
  if (!g) return null;
  if (/s-?gamut3[.\- ]?cine|sgamut3cine|s-log3-cine/.test(g)) return { gamut: "S-Gamut3.Cine", dvCS: "Sony S-Gamut3.Cine" };
  if (/s-?gamut3/.test(g)) return { gamut: "S-Gamut3", dvCS: "Sony S-Gamut3" };
  if (/s-?gamut/.test(g)) return { gamut: "S-Gamut", dvCS: "Sony S-Gamut" };
  if (/v-?gamut/.test(g)) return { gamut: "V-Gamut", dvCS: "Panasonic V-Gamut" };
  if (/awg4|wide ?gamut ?4/.test(g)) return { gamut: "ARRI Wide Gamut 4", dvCS: "ARRI Wide Gamut 4" };
  if (/awg3|wide ?gamut ?3|alexa ?wide/.test(g)) return { gamut: "ARRI Wide Gamut 3", dvCS: "ARRI Wide Gamut 3" };
  if (/cinema ?gamut/.test(g)) return { gamut: "Cinema Gamut", dvCS: "Canon Cinema Gamut" };
  if (/f-?gamut/.test(g)) return { gamut: "F-Gamut", dvCS: "Fujifilm F-Gamut" };
  if (/rwg|red ?wide ?gamut/.test(g)) return { gamut: "REDWideGamutRGB", dvCS: "REDWideGamutRGB" };
  if (/dci-?p3|display ?p3|\bp3\b/.test(g)) return { gamut: "P3", dvCS: "P3-D65" };
  if (/bt\.?2020|rec\.?2020|\b2020\b/.test(g)) return { gamut: "Rec.2020", dvCS: "Rec.2020" };
  if (/bt\.?709|rec\.?709|\b709\b/.test(g)) return { gamut: "Rec.709", dvCS: "Rec.709" };
  return null;
}
// transfer_characteristics как строка, но только если это НАЗВАНИЕ (BT.709/PQ/HLG…), а не hex-UL (шум).
function namedTransfer(vid, oth) {
  const tr = String(vid?.transfer_characteristics || rtmd(oth, "TransferCharacteristics_FirstFrame") || "");
  if (!tr || /^[0-9A-Fa-f]{12,}$/.test(tr)) return null;
  return tr;
}
// Гамут по умолчанию для бренда, если явного тега гамута нет (лог почти всегда пары со «своим» гамутом).
const VENDOR_GAMUT = {
  Sony: { gamut: "S-Gamut3", dvCS: "Sony S-Gamut3" },
  Panasonic: { gamut: "V-Gamut", dvCS: "Panasonic V-Gamut" },
  ARRI: { gamut: "ARRI Wide Gamut 3", dvCS: "ARRI Wide Gamut 3" },
  Canon: { gamut: "Cinema Gamut", dvCS: "Canon Cinema Gamut" },
  RED: { gamut: "REDWideGamutRGB", dvCS: "REDWideGamutRGB" },
  Fujifilm: { gamut: "F-Gamut", dvCS: "Fujifilm F-Gamut" },
  Nikon: { gamut: "Rec.2020", dvCS: "Rec.2020" },
  DJI: { gamut: "Rec.2020", dvCS: "Rec.2020" },
};

// Единый честный разбор цвета для ЛЮБОЙ камеры. Приоритет источников:
// capture-теги камеры (Sony AcquisitionRecord, Canon/Panasonic/Fuji/Nikon exif) -> transfer файла.
// Если сигнала гаммы НЕТ — возвращаем known=false и НЕ выдумываем Rec.709 (главный анти-баг).
function resolveColor(vid, oth, ex, acq) {
  acq = acq || {};
  const cands = [
    ["camera", acq.CaptureGammaEquation],
    ["camera", exifPick(ex, "CanonLogVersion")],
    ["camera", exifPick(ex, "PhotoStyle")],      // Panasonic V-Log
    ["camera", exifPick(ex, "FilmMode")],        // Fuji F-Log / F-Log2
    ["camera", exifPickFirst(ex, ["PictureControlBase", "PictureControlName"])], // Nikon
    ["camera", rtmd(oth, "CaptureGammaEquation_FirstFrame")],
    ["camera", exifPickFirst(ex, ["PictureProfile", "LogMode", "Gamma"])],
    ["file", namedTransfer(vid, oth)],
  ];
  let log = null, basis = null;
  for (const [b, s] of cands) { const r = logFromStr(s); if (r) { log = r; basis = b; break; } }

  const gmCands = [acq.CaptureColorPrimaries, acq.CaptureGammaEquation, rtmd(oth, "CaptureColorPrimaries_FirstFrame"),
    vid?.colour_primaries, rtmd(oth, "ColorPrimaries_FirstFrame")];
  let gut = null;
  for (const s of gmCands) { const r = gamutFromStr(s); if (r) { gut = r; break; } }
  if (log && !gut && VENDOR_GAMUT[log.vendor]) gut = VENDOR_GAMUT[log.vendor];

  let profileLabel = null;
  if (log) profileLabel = gut && gut.gamut && !/rec\.?709/i.test(log.gamma) ? `${log.gamma} · ${gut.gamut}` : log.gamma;
  return { profileLabel, dvCS: gut ? gut.dvCS : null, dvGamma: log ? log.dvGamma : null, basis, known: !!log };
}

// ======================= summary (keyed) =======================
function buildSummary(f) {
  const r = f.report;
  const tracks = miTracks(r);
  const gen = trackOf(tracks, "general"), vid = trackOf(tracks, "video");
  const aud = trackOf(tracks, "audio"), img = trackOf(tracks, "image");
  const oth = trackOf(tracks, "other");
  const ex = exifObj(r);
  const groups = [];
  const g = (titleKey, rows) => {
    const clean = rows.filter((x) => x[1] != null && x[1] !== "");
    if (clean.length) groups.push({ titleKey, rows: clean });
  };
  // Битрейт видео: mediainfo иногда завышает BitRate (путает основной поток XAVC S с вшитым прокси).
  // Если он выше общего по файлу — недостоверен: берём номинальный/максимальный.
  const vBitRaw = num(vid?.BitRate), oBit = num(gen?.OverallBitRate);
  const vBit = (vBitRaw != null && oBit != null && vBitRaw > oBit)
    ? (vid?.BitRate_Nominal || vid?.BitRate_Maximum || null) : vid?.BitRate;
  const mediaRows = [
    ["lblName", f.name],
    ["lblSize", `${fmtSize(r.size_bytes)} (${r.size_bytes} B)`],
    ["lblFormat", gen?.Format || img?.Format],
    ["lblDuration", gen?.Duration ? fmtDur(gen.Duration) : null],
    ["lblOverallBitrate", gen ? fmtBitrate(gen.OverallBitRate) : null],
    ["lblRecorded", gen?.Encoded_Date || gen?.File_Modified_Date],
  ];
  if (vid) mediaRows.push(
    ["lblCodec", vid.Format],
    ["lblProfile", vid.Format_Profile],
    ["lblResolution", vid.Width && vid.Height ? `${vid.Width}×${vid.Height}` : null],
    ["lblAspect", vid.DisplayAspectRatio],
    ["lblFps", vid.FrameRate ? `${num(vid.FrameRate)} (${vid.FrameRate_Mode || "?"})` : null],
    ["lblBitrate", fmtBitrate(vBit)],
  );
  if (img && !vid) mediaRows.push(
    ["lblResolution", img.Width && img.Height ? `${img.Width}×${img.Height}` : null],
    ["lblDepth", img.BitDepth ? `${img.BitDepth} bit` : null],
  );
  g("grpMedia", mediaRows);
  // Color science — colorist/DoP (primaries, transfer, log/gamma). Sony XAVC: из rtmd Other-трека.
  const prim = vid?.colour_primaries || rtmd(oth, "ColorPrimaries_FirstFrame");
  let transfer = vid?.transfer_characteristics || rtmd(oth, "TransferCharacteristics_FirstFrame");
  if (transfer && /^[0-9A-Fa-f]{8,}$/.test(String(transfer))) transfer = null; // hex UL — не показываем
  const acq = sonyAcq(f); // Sony AcquisitionRecord: истинная гамма/гамут съёмки
  const col = resolveColor(vid, oth, ex, acq); // честный разбор для любой камеры (не выдумывает 709)
  // Профиль/Log: сначала распознанная кривая, иначе — имя профиля камеры как есть (не гамма-догадка).
  const colorProfile = col.profileLabel
    || exifPickFirst(ex, ["PictureProfile", "PictureStyle", "CreativeStyle", "PhotoStyle", "FilmMode", "ProfileName"]);
  // Вшитый мониторинг-LUT (интендед-лук, напр. S-Log3→709) — колористу знать полезно.
  const monLut = String(exifPick(ex, "RelevantFilesRelatedToRel") || "").toUpperCase() === "LUT"
    ? exifPick(ex, "RelevantFilesRelatedToFile") : null;
  if (vid || prim) g("grpColor", [
    ["lblColorProfile", colorProfile],
    ["lblMonLut", monLut],
    ["lblPrimaries", prim],
    ["lblTransfer", transfer],
    ["lblRange", vid?.colour_range],
    ["lblChroma", vid?.ChromaSubsampling],
    ["lblDepth", vid?.BitDepth ? `${vid.BitDepth} bit` : null],
    ["lblResolveHint", col.known && col.dvCS && col.dvGamma ? `${col.dvCS} / ${col.dvGamma}` : null],
  ]);
  if (aud) g("grpAudioProduction", [
    ["lblCodec", aud.Format],
    ["lblChannels", aud.Channels ? `${aud.Channels}${aud.ChannelLayout ? ` (${aud.ChannelLayout})` : ""}` : null],
    ["lblSampleRate", fmtRate(aud.SamplingRate)],
    ["lblBitrate", fmtBitrate(aud.BitRate)],
    ["lblDepth", aud.BitDepth ? `${aud.BitDepth} bit` : null],
    ["lblTimecode", (oth && oth.TimeCode_FirstFrame) || exifPickFirst(ex, ["TimeCode", "StartTimecode", "TimeReference"])],
    ["lblScene", exifPick(ex, "Scene")],
    ["lblTake", exifPick(ex, "Take")],
    ["lblReel", exifPickFirst(ex, ["Reel", "TapeName", "Tape"])],
  ]);
  // Camera + lens — top colorist/DoP priority. Sony rtmd (oth.extra) первичен, exif — фолбэк.
  const iris = num(rtmd(oth, "IrisFNumber_FirstFrame"));
  const exifF = num(exifPick(ex, "FNumber"));
  const aperture = iris != null ? `f/${iris.toFixed(1)}` : (exifF != null ? `f/${exifF}` : exifPick(ex, "Aperture"));
  const saRaw = oth && oth.extra && oth.extra.ShutterSpeed_Angle_FirstFrame;
  const shutterAngle = saRaw ? `${saRaw}°` : null;
  const wb = rtmd(oth, "WhiteBalance_FirstFrame");
  const wbMode = rtmd(oth, "AutoWhiteBalanceMode_FirstFrame");
  const wbVal = wb ? `${wb}${wbMode ? ` (${wbMode})` : ""}` : exifPick(ex, "WhiteBalance");
  const camRows = [
    ["lblCamera", rtmd(oth, "CameraAttributes_FirstFrame") || [exifPick(ex, "Make"), exifPick(ex, "Model")].filter(Boolean).join(" ") || null],
    ["lblLens", exifPickFirst(ex, ["LensModel", "LensID", "LensSpecification", "LensInfo"])],
    ["lblFocus", rtmd(oth, "LensZoomActualFocalLength_FirstFrame") || exifPick(ex, "FocalLength")],
    ["lblAperture", aperture],
    ["lblIso", rtmd(oth, "ISOSensitivity_FirstFrame") || exifPickFirst(ex, ["ISO", "ISOSpeed"])],
    ["lblEi", rtmd(oth, "ExposureIndexofPhotoMeter_FirstFrame") || exifPickFirst(ex, ["ExposureIndex", "RecommendedExposureIndex"])],
    ["lblShutter", shutterAngle || rtmd(oth, "ShutterSpeed_Time_FirstFrame") || exifPickFirst(ex, ["ShutterSpeed", "ExposureTime"])],
    ["lblWb", wbVal],
  ];
  if (camRows.some((r) => r[1] != null && r[1] !== "")) g("grpCamera", camRows);
  return groups;
}

// ======================= loudness (EBU R128, on demand) =======================
function loudCardHtml(f) {
  const hasAudio = !!trackOf(miTracks(f.report), "audio");
  const title = esc(t("grpLoud"));
  if (!hasAudio)
    return `<div class="card"><h3>${title}</h3><div class="load">${t("loudNoAudio")}</div></div>`;
  const L = f.loud;
  let body;
  if (f.loudBusy) {
    body = `<div class="load">${t("loudMeasuring")}</div>`;
  } else if (L && L.ok) {
    const tp = L.true_peak;
    const rows = [
      ["lblIntegrated", L.integrated != null ? `${L.integrated.toFixed(1)} LUFS` : "—", ""],
      ["lblTruePeak", tp != null ? `${tp.toFixed(1)} dBTP` : "—", tp != null && tp > -1 ? "loud-warn" : "loud-ok"],
      ["lblLra", L.lra != null ? `${L.lra.toFixed(1)} LU` : "—", ""],
    ];
    body = `<dl>` + rows.map(([k, v, c]) => `<dt>${esc(t(k))}</dt><dd class="${c}">${esc(v)}</dd>`).join("") + `</dl>` +
      `<div class="loud-hint">${esc(t("loudTargets"))}</div>`;
  } else if (L && !L.ok) {
    body = `<div class="err">⚠ ${esc(L.error || "")}</div>` +
      `<button class="loud-btn" id="loud-btn" type="button">${esc(t("loudMeasure"))}</button>`;
  } else {
    body = `<button class="loud-btn" id="loud-btn" type="button">${esc(t("loudMeasure"))}</button>` +
      `<div class="loud-hint">${esc(t("loudTargets"))}</div>`;
  }
  return `<div class="card loud-card"><h3>${title}</h3>${body}</div>`;
}
function wireLoud(f) {
  const b = document.getElementById("loud-btn");
  if (b) b.onclick = () => measureLoudness(f);
}
async function measureLoudness(f) {
  if (f.loudBusy) return;
  f.loudBusy = true; render();
  try {
    f.loud = await invoke("measure_loudness", { path: f.path });
  } catch (e) {
    f.loud = { ok: false, error: typeof e === "string" ? e : "measure error" };
  }
  f.loudBusy = false; render();
  logEvent(`loudness measured ok=${f.loud && f.loud.ok}`);
}

// ======================= checksum + frame scan (on demand) =======================
function hashCardHtml(f) {
  const title = esc(t("grpChecksum"));
  const hint = `<div class="loud-hint">${esc(t("hashHint"))}</div>`;
  let body;
  if (f.hashBusy) body = `<div class="load">${t("hashBusy")}</div>`;
  else if (f.hash && f.hash.ok)
    body = `<div class="dcp-raw">${esc(f.hash.hash)}</div><button class="loud-btn" id="hash-copy" type="button">${esc(t("hashCopy"))}</button>`;
  else if (f.hash && !f.hash.ok)
    body = `<div class="err">⚠ ${esc(f.hash.error)}</div><button class="loud-btn" id="hash-btn" type="button">${esc(t("hashBtn"))}</button>`;
  else body = `<button class="loud-btn" id="hash-btn" type="button">${esc(t("hashBtn"))}</button>`;
  return `<div class="card"><h3>${title}</h3>${body}${hint}</div>`;
}
function scanCardHtml(f) {
  if (!trackOf(miTracks(f.report), "video")) return ""; // скан только для видео
  const title = esc(t("grpScan"));
  let body;
  if (f.scanBusy) body = `<div class="load">${t("scanBusy")}</div>`;
  else if (f.scan && f.scan.ok) {
    const segs = f.scan.segments || [];
    if (!segs.length) body = `<div style="color:#5fd08a;font-size:13px">✔ ${esc(t("scanNone"))}</div>`;
    else {
      const rows = segs.map((s) => {
        const k = s.kind === "black" ? t("scanBlack") : t("scanFreeze");
        const rng = s.end != null ? `${fmtDur(s.start)} – ${fmtDur(s.end)}` : `${fmtDur(s.start)} →`;
        return `<li class="dcp-chk dcp-warn"><span class="dcp-ci">⚠</span><span class="dcp-scope">${esc(k)}</span><span>${esc(rng)}</span></li>`;
      }).join("");
      body = `<ul class="dcp-checks">${rows}</ul>`;
    }
    body += `<button class="loud-btn" id="scan-btn" type="button" style="margin-top:8px">${esc(t("scanBtn"))}</button>`;
  } else if (f.scan && !f.scan.ok)
    body = `<div class="err">⚠ ${esc(f.scan.error)}</div><button class="loud-btn" id="scan-btn" type="button">${esc(t("scanBtn"))}</button>`;
  else body = `<button class="loud-btn" id="scan-btn" type="button">${esc(t("scanBtn"))}</button><div class="loud-hint">${esc(t("scanHint"))}</div>`;
  return `<div class="card dcp-checks-card"><h3>${title}</h3>${body}</div>`;
}
function wireHashScan(f) {
  const hb = document.getElementById("hash-btn"); if (hb) hb.onclick = () => computeHash(f);
  const hc = document.getElementById("hash-copy");
  if (hc) hc.onclick = async () => { if (f.hash && f.hash.ok) { await copyText(f.hash.hash); hc.textContent = t("copied"); setTimeout(() => { hc.textContent = t("hashCopy"); }, 1200); } };
  const sb = document.getElementById("scan-btn"); if (sb) sb.onclick = () => scanFrames(f);
}
async function computeHash(f) {
  if (f.hashBusy) return;
  f.hashBusy = true; render();
  try { f.hash = await invoke("hash_file", { path: f.path }); }
  catch (e) { f.hash = { ok: false, error: typeof e === "string" ? e : "hash error" }; }
  f.hashBusy = false; render();
  logEvent(`hash ok=${f.hash && f.hash.ok}`);
}
async function scanFrames(f) {
  if (f.scanBusy) return;
  f.scanBusy = true; render();
  try { f.scan = await invoke("scan_frames", { path: f.path }); }
  catch (e) { f.scan = { ok: false, error: typeof e === "string" ? e : "scan error" }; }
  f.scanBusy = false; render();
  logEvent(`scan ok=${f.scan && f.scan.ok} segs=${f.scan && f.scan.segments ? f.scan.segments.length : 0}`);
}

// ======================= DCP validation (DCNC + DCI + Netflix) =======================
const DCNC_TYPES = ["FTR", "TLR", "TSR", "TST", "RTG", "SHR", "ADV", "PSA", "XSN", "PRO", "POL", "EPS", "CLP"];
const DCNC_ASPECT = /^(F|S|C)(-?\d{2,3})?$/;
const DCNC_RES = /^(2K|4K)$/i;
const DCNC_AUDIO = /^(10|20|51|71|IAB|ATMOS|MOS)$/i;
// Служебные/шаблонные имена, которые кинотеатры отклоняют.
const DCP_PLACEHOLDER = /^(untitled.*|composition[ _-]?title|dcp[ _-]*master|new[ _-]?project|master|final|test|export|output|render|sequence\d*|timeline\d*|project\d*)$/i;

function parseDcnc(title) {
  const segs = title.split("_");
  const f = { title: segs[0] || null, contentType: null, aspect: null, audio: null, resolution: null, standard: null, package: null, date: null, segs };
  for (const s of segs.slice(1)) {
    const S = s.toUpperCase();
    if (!f.contentType && DCNC_TYPES.some((ct) => S === ct || S.startsWith(ct + "-"))) { f.contentType = s; continue; }
    if (!f.aspect && DCNC_ASPECT.test(S)) { f.aspect = s; continue; }
    if (!f.resolution && DCNC_RES.test(S)) { f.resolution = s; continue; }
    if (!f.standard && (S === "SMPTE" || S === "IOP")) { f.standard = s; continue; }
    if (!f.package && (S === "OV" || S === "VF")) { f.package = s; continue; }
    if (!f.date && /^\d{8}$/.test(S)) { f.date = s; continue; }
    if (!f.audio && DCNC_AUDIO.test(S)) { f.audio = s; continue; }
  }
  return f;
}
function dciResName(w, h) {
  return ({ "2048x1080": "2K Full", "1998x1080": "2K Flat", "2048x858": "2K Scope",
            "4096x2160": "4K Full", "3996x2160": "4K Flat", "4096x1716": "4K Scope" })[`${w}x${h}`] || null;
}

function evalDcp(d) {
  const checks = [];
  const add = (level, scope, text) => checks.push({ level, scope, text });
  const SNAME = L("Имя", "Naming"), SSTR = L("Структура", "Structure");
  if (!d.is_dcp) {
    add("err", SSTR, L("Не найден CPL — это не похоже на DCP-пакет.", "No CPL found — not a DCP package."));
    return { checks, fields: null, verdict: "fail" };
  }
  // структура
  d.has_assetmap ? add("ok", SSTR, "ASSETMAP ✓") : add("err", SSTR, L("Нет ASSETMAP — плеер не соберёт пакет", "Missing ASSETMAP"));
  d.has_pkl ? add("ok", SSTR, "PKL ✓") : add("err", SSTR, L("Нет PKL (packing list) — пакет неполный", "Missing PKL"));
  d.has_volindex ? add("ok", SSTR, "VOLINDEX ✓") : add("warn", SSTR, L("Нет VOLINDEX", "Missing VOLINDEX"));
  d.mxf_count > 0 ? add("ok", SSTR, `MXF: ${d.mxf_count}`) : add("err", SSTR, L("Нет MXF-эссенций (видео/звук)", "No MXF essence files"));

  // наименование
  const titles = d.cpls.map((c) => c.content_title).filter(Boolean);
  const dups = [...new Set(titles.filter((t2, i) => titles.indexOf(t2) !== i))];
  if (dups.length) add("err", SNAME, L("Дубли имён CPL: ", "Duplicate CPL titles: ") + dups.join(", "));

  let fields = null;
  const c0 = d.cpls[0];
  if (c0) {
    const title = c0.content_title;
    if (!title) add("err", SNAME, L("У CPL пустой ContentTitleText", "Empty ContentTitleText"));
    else {
      fields = parseDcnc(title);
      if (/[^A-Za-z0-9_-]/.test(title)) {
        const bad = [...new Set(title.match(/[^A-Za-z0-9_-]/g))].map((c) => (c === " " ? "␠" : c)).join(" ");
        add("err", SNAME, L(`Недопустимые символы (${bad}). Только латиница, цифры, «_» и «-» — без пробелов и кириллицы.`,
                            `Illegal characters (${bad}). Latin letters, digits, «_» and «-» only.`));
      } else add("ok", SNAME, L("Только латиница/цифры/_/-", "Latin/digits/_/- only"));
      const first = title.split("_")[0] || "";
      if (first.length < 2)
        add("err", SNAME, L("В начале имени нет названия проекта (пустое поле Title) — кинотеатр отклонит.",
                            "No project title at the start (empty Title field) — cinemas reject this."));
      if (DCP_PLACEHOLDER.test(title) || DCP_PLACEHOLDER.test(first))
        add("err", SNAME, L(`Служебное/шаблонное имя «${title}» — кинотеатр отклонит. Дай осмысленное название.`,
                            `Placeholder name «${title}» — cinemas reject these.`));
      if (fields.segs.length < 6)
        add("warn", SNAME, L(`Мало полей DCNC (${fields.segs.length}). Формат: Title_Type_Aspect_Lang_Terr_Audio_Res_Studio_Date_Facility_Standard_Package`,
                             `Too few DCNC fields (${fields.segs.length}).`));
      else add("ok", SNAME, L(`Полей: ${fields.segs.length}`, `Fields: ${fields.segs.length}`));
      if (!fields.contentType) add("warn", SNAME, L("Не распознан тип (FTR/TLR/SHR…)", "Content type not recognized"));
      if (!fields.resolution) add("warn", SNAME, L("Нет 2K/4K в имени", "No 2K/4K in name"));
      const hasNet = fields.segs.some((s) => s.toUpperCase() === "NET");
      hasNet ? add("ok", "Netflix", L("Студийный код NET есть", "Studio code NET present"))
             : add("info", "Netflix", L("Для Netflix студийный код должен быть NET", "Netflix requires studio code NET"));
    }
  }

  // техника из рилов
  let maxReel = 0; const fpsSet = new Set();
  d.cpls.forEach((c) => c.reels.forEach((r) => { if (r.duration_sec) maxReel = Math.max(maxReel, r.duration_sec); if (r.fps) fpsSet.add(Math.round(r.fps)); }));
  if (maxReel > 0) {
    if (maxReel > 22 * 60) {
      add("warn", "DCI", L(`Длинный рил ${fmtDur(maxReel)} — DCI советует ≤22 мин`, `Long reel ${fmtDur(maxReel)} — DCI recommends ≤22 min`));
      add("err", "Netflix", L(`Рил ${fmtDur(maxReel)} > 22 мин — Netflix отклонит`, `Reel ${fmtDur(maxReel)} > 22 min — Netflix rejects`));
    } else add("ok", "DCI", L("Длина рилов ≤22 мин", "Reel length ≤22 min"));
  }
  [...fpsSet].forEach((fp) => { if (![24, 25, 30, 48, 50, 60].includes(fp)) add("warn", "DCI", L(`Нестандартный кадр ${fp} fps`, `Non-standard ${fp} fps`)); });
  if (fpsSet.size > 1) add("warn", "DCI", L("Разный framerate у рилов: ", "Mixed frame rates: ") + [...fpsSet].join(", "));
  if (d.cpls.some((c) => c.standard === "Interop") && [...fpsSet].some((fp) => fp !== 24))
    add("warn", "DCI", L("Interop-DCP должен быть 24 fps", "Interop DCP must be 24 fps"));

  // картинка (проба MXF)
  const p = d.picture;
  if (p) {
    if (p.color_space)
      /xyz/i.test(p.color_space) ? add("ok", "DCI", L("Цвет XYZ", "Color XYZ"))
                                 : add("warn", "DCI", L(`Цвет ${p.color_space} — DCI ожидает XYZ`, `Color ${p.color_space} — DCI expects XYZ`));
    if (p.width && p.height) {
      const dci = dciResName(p.width, p.height);
      dci ? add("ok", "DCI", `${p.width}×${p.height} (${dci})`)
          : add("warn", "DCI", L(`${p.width}×${p.height} — не DCI-контейнер`, `${p.width}×${p.height} — not a DCI container`));
      if (fields && fields.resolution) {
        const named = fields.resolution.toUpperCase();
        const real = (p.width >= 3000 || p.height >= 1700) ? "4K" : "2K";
        if (named !== real) add("warn", SNAME, L(`В имени ${named}, по факту ${real}`, `Name says ${named}, essence is ${real}`));
      }
    }
    const br = p.bit_rate != null ? num(p.bit_rate) / 1e6 : null;
    if (br != null)
      br > 250 ? add("warn", "Netflix", L(`Битрейт ${br.toFixed(0)} Мбит/с > 250 (лимит Netflix SDR)`, `Bitrate ${br.toFixed(0)} Mb/s > 250 (Netflix SDR limit)`))
               : add("ok", "Netflix", `≤250 Mb/s (${br.toFixed(0)})`);
  }
  const a = d.audio;
  if (a && a.channels != null)
    (a.channels % 2 === 0) ? add("ok", "Netflix", L(`Аудиоканалов ${a.channels} (чётно)`, `${a.channels} audio ch (even)`))
                           : add("warn", "Netflix", L(`Каналов ${a.channels} — Netflix требует чётное`, `${a.channels} ch — Netflix requires even`));

  const enc = d.cpls.some((c) => c.encrypted);
  add("info", "Netflix", enc ? L("Зашифрован — для показа нужен KDM/DKDM", "Encrypted — needs KDM/DKDM")
                             : L("Без шифрования (open DCP)", "Unencrypted (open DCP)"));

  const hasErr = checks.some((c) => c.level === "err");
  const hasWarn = checks.some((c) => c.level === "warn");
  return { checks, fields, verdict: hasErr ? "fail" : hasWarn ? "warn" : "pass" };
}

function buildDcpHtml(f) {
  const d = f.report;
  const res = evalDcp(d);
  f.dcpEval = res; // для markdown-экспорта
  const cards = [];

  const vClass = { pass: "dcp-pass", warn: "dcp-warn", fail: "dcp-fail" }[res.verdict];
  const vText = { pass: t("dcpPass"), warn: t("dcpWarn"), fail: t("dcpFail") }[res.verdict];
  const nErr = res.checks.filter((c) => c.level === "err").length;
  const nW = res.checks.filter((c) => c.level === "warn").length;
  cards.push(`<div class="card dcp-verdict ${vClass}"><h3>${esc(t("dcpVerdict"))}</h3>` +
    `<div class="dcp-badge">${esc(vText)}</div><div class="dcp-vsub">✖ ${nErr} · ⚠ ${nW}</div></div>`);

  if (d.cpls[0]) {
    const fr = res.fields;
    let inner = `<div class="dcp-raw">${esc(d.cpls[0].content_title || "—")}</div>`;
    if (fr) {
      const rows = [[t("dcpTitle"), fr.title], [t("dcpType"), fr.contentType], [t("dcpAspect"), fr.aspect],
        [t("dcpAudioF"), fr.audio], [t("dcpResF"), fr.resolution], [t("dcpStd"), fr.standard || d.cpls[0].standard],
        [t("dcpPkg"), fr.package], [t("dcpDate"), fr.date]].filter((r) => r[1]);
      inner += `<dl>${rows.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join("")}</dl>`;
    }
    cards.push(`<div class="card"><h3>${esc(t("dcpNaming"))}</h3>${inner}</div>`);
  }

  const kinds = ["ASSETMAP", "VOLINDEX", "PKL", "CPL", "MXF"];
  const presMap = { ASSETMAP: d.has_assetmap, VOLINDEX: d.has_volindex, PKL: d.has_pkl, CPL: d.cpl_count > 0, MXF: d.mxf_count > 0 };
  const present = kinds.map((k) => `${presMap[k] ? "✔" : "✖"} ${k}`).join("&nbsp;&nbsp;");
  const flist = d.files.slice().sort((x, y) => x.kind.localeCompare(y.kind))
    .map((x) => `<div class="dcp-file"><span>${esc(x.name)}</span><span class="dim">${esc(x.kind)} · ${fmtSize(x.size)}</span></div>`).join("");
  cards.push(`<div class="card"><h3>${esc(t("dcpStruct"))}</h3><div class="dcp-present">${present}</div>` +
    `<div class="dcp-files">${flist}</div><div class="dim" style="margin-top:6px">${L("Всего", "Total")}: ${fmtSize(d.total_size)}</div></div>`);

  const p = d.picture, a = d.audio;
  const mrows = [];
  if (p) {
    mrows.push([t("lblResolution"), p.width && p.height ? `${p.width}×${p.height}` : null]);
    mrows.push([t("lblCodec"), p.format]); mrows.push([t("lblColor"), p.color_space]);
    mrows.push([t("lblFps"), p.frame_rate]); mrows.push([t("lblDepth"), p.bit_depth ? `${p.bit_depth} bit` : null]);
    mrows.push([t("lblBitrate"), p.bit_rate ? fmtBitrate(p.bit_rate) : null]);
  }
  if (a) {
    mrows.push([t("lblChannels"), a.channels]); mrows.push([t("lblSampleRate"), a.sample_rate ? fmtRate(a.sample_rate) : null]);
    mrows.push([t("lblCodec") + " (audio)", a.format]); mrows.push([t("lblDepth") + " (audio)", a.bit_depth ? `${a.bit_depth} bit` : null]);
  }
  const reels = d.cpls[0]?.reels || [];
  if (reels.length) {
    const rt = reels.map((r, i) => `#${i + 1} ${r.duration_sec ? fmtDur(r.duration_sec) : "?"}${r.fps ? ` @${Math.round(r.fps)}` : ""}${r.aspect ? ` ${r.aspect}` : ""}`).join("  ·  ");
    mrows.push([t("dcpReels"), `${reels.length} — ${rt}`]);
  }
  const mclean = mrows.filter((r) => r[1] != null && r[1] !== "");
  if (mclean.length)
    cards.push(`<div class="card"><h3>${esc(t("dcpMedia"))}</h3><dl>${mclean.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(String(v))}</dd>`).join("")}</dl></div>`);

  const ico = { ok: "✔", info: "•", warn: "⚠", err: "✖" };
  const ordr = { err: 0, warn: 1, info: 2, ok: 3 };
  const sorted = res.checks.slice().sort((x, y) => ordr[x.level] - ordr[y.level]);
  const items = sorted.map((c) => `<li class="dcp-chk dcp-${c.level}"><span class="dcp-ci">${ico[c.level]}</span>` +
    `<span class="dcp-scope">${esc(c.scope)}</span><span>${esc(c.text)}</span></li>`).join("");
  cards.push(`<div class="card dcp-checks-card"><h3>${esc(t("dcpChecks"))}</h3><ul class="dcp-checks">${items}</ul></div>`);

  return cards.join("");
}

function buildDcpMarkdown(f) {
  const d = f.report;
  const res = f.dcpEval || evalDcp(d);
  const now = new Date().toISOString().replace("T", " ").slice(0, 19);
  let md = `# DCP Report — ${f.name}\n\n- **Folder:** \`${f.path}\`\n- **Analyzed:** ${now}\n- **Verdict:** ${res.verdict.toUpperCase()}\n\n`;
  if (d.cpls[0]) {
    md += `## Naming (DCNC)\n\n\`${d.cpls[0].content_title || "—"}\`\n\n`;
    if (res.fields) {
      const fr = res.fields;
      const rows = [["Title", fr.title], ["Type", fr.contentType], ["Aspect", fr.aspect], ["Audio", fr.audio],
        ["Resolution", fr.resolution], ["Standard", fr.standard || d.cpls[0].standard], ["Package", fr.package], ["Date", fr.date]].filter((r) => r[1]);
      if (rows.length) md += `| Field | Value |\n|---|---|\n` + rows.map(([k, v]) => `| ${k} | ${v} |\n`).join("") + `\n`;
    }
  }
  md += `## Structure\n\n- ASSETMAP: ${d.has_assetmap ? "yes" : "NO"}\n- PKL: ${d.has_pkl ? "yes" : "NO"}\n- VOLINDEX: ${d.has_volindex ? "yes" : "no"}\n- CPL: ${d.cpl_count}\n- MXF: ${d.mxf_count}\n- Total size: ${fmtSize(d.total_size)}\n\n`;
  const p = d.picture, a = d.audio;
  if (p || a) {
    md += `## Essence (MXF)\n\n`;
    if (p) { md += `- Picture: ${p.width || "?"}×${p.height || "?"} ${p.format || ""} ${p.color_space || ""} ${p.frame_rate || ""}\n`.replace(/ +/g, " "); if (p.bit_rate) md += `- Bitrate: ${fmtBitrate(p.bit_rate)}\n`; }
    if (a) md += `- Audio: ${a.channels ?? "?"} ch ${a.format || ""} ${a.sample_rate ? fmtRate(a.sample_rate) : ""} ${a.bit_depth ? a.bit_depth + "bit" : ""}\n`.replace(/ +/g, " ");
    md += `\n`;
  }
  md += `## Checks\n\n`;
  const tag = { ok: "OK", info: "i", warn: "WARN", err: "FAIL" };
  const ordr = { err: 0, warn: 1, info: 2, ok: 3 };
  res.checks.slice().sort((x, y) => ordr[x.level] - ordr[y.level]).forEach((c) => { md += `- [${tag[c.level]}] (${c.scope}) ${c.text}\n`; });
  return md + `\n`;
}

// ======================= delivery spec check (regular files) =======================
// Реальные опубликованные спеки. Числа с источниками — не выдумка. Loudness сверяется
// с уже измеренной (measure_loudness / ffmpeg ebur128 = полнопрограммная BS.1770, НЕ диалог-гейт).
const SPEC_PROFILES = {
  "ebu-r128": {
    label: { en: "EBU R128 · EU broadcast", ru: "EBU R128 · эфир ЕС" },
    // EBU R128-2020 §h: −23.0 LUFS ±1.0 LU · §m: True Peak ≤ −1 dBTP
    loud: { target: -23.0, tol: 1.0, tpMax: -1.0 },
    audio: { sampleRate: 48000 }, video: { stdFps: true, cfr: true },
  },
  "atsc-a85": {
    label: { en: "ATSC A/85 · US broadcast", ru: "ATSC A/85 · эфир США" },
    // ATSC A/85 / CALM Act: −24 LKFS ±2 · TP −2 dBTP (US-broadcast common ceiling)
    loud: { target: -24.0, tol: 2.0, tpMax: -2.0 },
    audio: { sampleRate: 48000 }, video: { stdFps: true, cfr: true },
  },
  "netflix": {
    label: { en: "Netflix · program loudness", ru: "Netflix · программная громкость" },
    // Netflix Sound Mix v1.6 §4.1: program −24 LKFS ±2 (BS.1770-4) · §4.2 TP −2 dBTP · LRA 4–18 LU
    loud: {
      target: -24.0, tol: 2.0, tpMax: -2.0, lraMin: 4, lraMax: 18,
      note: {
        en: "Dialog-gated −27 LKFS needs a dialog-gated meter; we measure full-program loudness (Netflix program spec).",
        ru: "Диалог-гейт −27 LKFS требует диалог-гейт-метра; мы меряем полнопрограммную громкость (программная спека Netflix).",
      },
    },
    audio: { sampleRate: 48000 }, video: { stdFps: true, cfr: true },
  },
  "social": {
    label: { en: "YouTube / social media · −14 LUFS", ru: "YouTube / соцсети · −14 LUFS" },
    // Playback normalization: YouTube/Spotify/TikTok ≈ −14 LUFS. Ориентир, не жёсткий reject.
    loud: {
      target: -14.0, tol: 1.0, tpMax: -1.0, soft: true,
      note: {
        en: "Platform playback normalization target — a guideline, not a hard reject.",
        ru: "Цель нормализации плеера платформы — ориентир, не жёсткий отказ.",
      },
    },
  },
};

// Пользовательский профиль из localStorage (Фаза 4). Один именованный набор порогов.
function customSpec() {
  try {
    const j = JSON.parse(localStorage.getItem("customSpec") || "null");
    if (!j || j.target == null || j.tol == null) return null;
    return {
      label: { en: `Custom · ${j.name || "profile"}`, ru: `Свой · ${j.name || "профиль"}` },
      loud: { target: j.target, tol: j.tol, tpMax: (j.tpMax === "" || j.tpMax == null) ? null : j.tpMax, soft: false },
      audio: j.sampleRate ? { sampleRate: j.sampleRate } : null,
      video: j.checkFps ? { stdFps: true, cfr: true } : null,
      custom: true,
    };
  } catch { return null; }
}
const getProfile = (id) => (id === "custom" ? customSpec() : SPEC_PROFILES[id]);

// Возвращает { checks, verdict, profile } — та же форма, что evalDcp (переиспользуем рендер/CSS).
function evalSpec(f, id) {
  const prof = getProfile(id);
  if (!prof) return { checks: [], verdict: "pass", profile: { label: { en: "", ru: "" } } };
  const checks = [];
  const add = (level, scope, text) => checks.push({ level, scope, text });
  const SLOUD = "Loudness", SAUD = L("Звук", "Audio"), SVID = L("Видео", "Video");
  const tracks = miTracks(f.report);
  const vid = trackOf(tracks, "video"), aud = trackOf(tracks, "audio");

  if (prof.loud) {
    const Ld = f.loud;
    if (!aud) add("warn", SLOUD, L("Нет аудиодорожки для проверки громкости", "No audio track to check loudness"));
    else if (!Ld || !Ld.ok) add("warn", SLOUD, L("Громкость не измерена — нажми «Измерить громкость» ниже", "Loudness not measured — press “Measure loudness” below"));
    else {
      const { target, tol, tpMax, soft, lraMin, lraMax } = prof.loud;
      const I = Ld.integrated, tp = Ld.true_peak, lra = Ld.lra;
      if (I != null) {
        const dev = I - target;
        if (Math.abs(dev) <= tol) add("ok", SLOUD, `${I.toFixed(1)} LUFS ✓ (${target}±${tol})`);
        else add(soft ? "warn" : "err", SLOUD, L(`${I.toFixed(1)} LUFS — цель ${target}±${tol} (${dev > 0 ? "+" : ""}${dev.toFixed(1)} LU)`,
                                                  `${I.toFixed(1)} LUFS — target ${target}±${tol} (${dev > 0 ? "+" : ""}${dev.toFixed(1)} LU)`));
      }
      if (tp != null && tpMax != null) {
        if (tp <= tpMax) add("ok", SLOUD, `${tp.toFixed(1)} dBTP ✓ (≤${tpMax})`);
        else add("err", SLOUD, L(`True Peak ${tp.toFixed(1)} dBTP > ${tpMax} — клиппинг, QC отклонит`, `True Peak ${tp.toFixed(1)} dBTP > ${tpMax} — clipping, QC rejects`));
      }
      if (lra != null && lraMin != null)
        (lra >= lraMin && lra <= lraMax) ? add("ok", SLOUD, `LRA ${lra.toFixed(1)} LU ✓ (${lraMin}–${lraMax})`)
                                         : add("warn", SLOUD, L(`LRA ${lra.toFixed(1)} LU вне ${lraMin}–${lraMax}`, `LRA ${lra.toFixed(1)} LU outside ${lraMin}–${lraMax}`));
    }
    if (prof.loud.note) add("info", SLOUD, L(prof.loud.note.ru, prof.loud.note.en));
  }

  if (prof.audio && aud) {
    const sr = num(aud.SamplingRate);
    if (sr != null)
      sr === prof.audio.sampleRate ? add("ok", SAUD, `${(sr / 1000).toFixed(1)} kHz ✓`)
                                   : add("warn", SAUD, L(`Частота ${(sr / 1000).toFixed(1)} kHz — стандарт ${prof.audio.sampleRate / 1000} kHz`,
                                                         `Sample rate ${(sr / 1000).toFixed(1)} kHz — expected ${prof.audio.sampleRate / 1000} kHz`));
  }

  if (prof.video && vid) {
    const fps = num(vid.FrameRate);
    const STD = [23.976, 24, 25, 29.97, 30, 48, 50, 59.94, 60];
    if (prof.video.stdFps && fps != null)
      STD.some((s) => Math.abs(s - fps) < 0.05) ? add("ok", SVID, `${fps} fps ✓`)
                                                : add("warn", SVID, L(`Нестандартный кадр ${fps} fps`, `Non-standard ${fps} fps`));
    if (prof.video.cfr && vid.FrameRate_Mode)
      /vfr|variable/i.test(vid.FrameRate_Mode)
        ? add("err", SVID, L("Переменный фреймрейт (VFR) — рассинхрон при сдаче, нужен CFR", "Variable frame rate (VFR) — sync drift on delivery, needs CFR"))
        : add("ok", SVID, L("Постоянный фреймрейт (CFR)", "Constant frame rate (CFR)"));
  }

  const hasErr = checks.some((c) => c.level === "err");
  const hasWarn = checks.some((c) => c.level === "warn");
  return { checks, verdict: hasErr ? "fail" : hasWarn ? "warn" : "pass", profile: prof };
}

// Вердикт-карточка + чеклист — переиспользуют DCP-классы (traffic-light уже стилизован).
function buildSpecCards(f, id, snapshot = null) {
  const res = snapshot || evalSpec(f, id);
  f.specEval = res;
  const vClass = { pass: "dcp-pass", warn: "dcp-warn", fail: "dcp-fail" }[res.verdict];
  const vText = { pass: t("dcpPass"), warn: t("dcpWarn"), fail: t("dcpFail") }[res.verdict];
  const nErr = res.checks.filter((c) => c.level === "err").length;
  const nW = res.checks.filter((c) => c.level === "warn").length;
  const pl = res.profile.label[lang] || res.profile.label.en;
  const cards = [];
  cards.push(`<div class="card dcp-verdict ${vClass}"><h3>${esc(t("specVerdictTitle"))} · ${esc(pl)}</h3>` +
    `<div class="dcp-badge">${esc(vText)}</div><div class="dcp-vsub">✖ ${nErr} · ⚠ ${nW}</div></div>`);
  const ico = { ok: "✔", info: "•", warn: "⚠", err: "✖" };
  const ordr = { err: 0, warn: 1, info: 2, ok: 3 };
  const items = res.checks.slice().sort((x, y) => ordr[x.level] - ordr[y.level])
    .map((c) => `<li class="dcp-chk dcp-${c.level}"><span class="dcp-ci">${ico[c.level]}</span>` +
      `<span class="dcp-scope">${esc(c.scope)}</span><span>${esc(c.text)}</span></li>`).join("");
  cards.push(`<div class="card dcp-checks-card"><h3>${esc(t("specChecks"))}</h3><ul class="dcp-checks">${items}</ul></div>`);
  return cards.join("");
}

function buildChecksHtml(f) {
  const hasProfile = specProfile !== "none" && !!getProfile(specProfile);
  const hasRun = hasProfile && f.specRunProfile === specProfile;
  const profileLabel = hasProfile && (getProfile(specProfile).label[lang] || getProfile(specProfile).label.en);
  const result = hasRun ? `<div class="checks-results">${buildSpecCards(f, specProfile, f.specResult)}</div>`
    : `<div class="card checks-idle">${hasProfile ? `${esc(t("checksReady"))} <strong>${esc(profileLabel)}</strong>` : esc(t("checksIdle"))}</div>`;
  return `<section class="checks-intro">
      <h2>${esc(t("checksTitle"))}</h2>
      <p>${esc(t("checksProfileHint"))}</p>
    </section>
    <div class="card checks-config">
      <div class="checks-field">
        <label for="spec-profile">${esc(t("specLabel"))}</label>
        <span class="select-control"><select id="spec-profile"></select></span>
      </div>
      <button id="spec-run" class="loud-btn checks-run" type="button"${hasProfile ? "" : " disabled"}>${esc(t("checksRun"))}</button>
      <button id="spec-edit" class="set-btn checks-edit" type="button">${esc(t("specEditBtn"))}</button>
    </div>
    ${result}
    <section class="checks-tools">
      <h2>${esc(t("checksTechnical"))}</h2>
      <p>${esc(t("checksTechnicalHint"))}</p>
      <div class="checks-tools-grid">${loudCardHtml(f)}${hashCardHtml(f)}${scanCardHtml(f)}</div>
    </section>`;
}

function fillSpecOptions() {
  const sel = document.getElementById("spec-profile");
  if (!sel) return;
  const opts = [`<option value="none">${esc(t("specNone"))}</option>`];
  for (const [id, p] of Object.entries(SPEC_PROFILES))
    opts.push(`<option value="${id}">${esc(p.label[lang] || p.label.en)}</option>`);
  const cs = customSpec();
  if (cs) opts.push(`<option value="custom">${esc(cs.label[lang] || cs.label.en)}</option>`);
  sel.innerHTML = opts.join("");
  // выбранный custom мог быть удалён — откатываем на none
  if (specProfile === "custom" && !cs) specProfile = "none";
  sel.value = specProfile;
}

// ======================= ffprobe → readable (MediaInfo-like) =======================
function humanizeKey(k) {
  const s = k.replace(/_/g, " ");
  return s.charAt(0).toUpperCase() + s.slice(1);
}
function ffVal(key, v) {
  const s = String(v);
  if (key === "bit_rate" || key === "max_bit_rate") return fmtBitrate(v) || s;
  if (key === "duration") return fmtDur(v);
  if (key === "size") return fmtSize(v);
  if (key === "sample_rate") return fmtRate(v) || s;
  return s;
}
function ffSection(title, pairs) {
  const rows = pairs.filter(([, v]) => v != null && v !== "" && String(v) !== "N/A");
  if (!rows.length) return "";
  const w = Math.min(30, Math.max(...rows.map(([k]) => k.length)));
  const body = rows.map(([k, v]) => `  ${k.padEnd(w)} : ${v}`).join("\n");
  return `${title}\n${body}`;
}
function formatFfprobe(jsonStr) {
  let j;
  try { j = JSON.parse(jsonStr); } catch { return null; }
  if (!j || (!j.format && !j.streams)) return null;
  const blocks = [];
  if (j.format) {
    const f = j.format;
    const gen = [
      ["Format", f.format_long_name ? `${f.format_long_name} (${f.format_name})` : f.format_name],
      ["Duration", f.duration != null ? fmtDur(f.duration) : null],
      ["File size", f.size != null ? fmtSize(f.size) : null],
      ["Overall bit rate", fmtBitrate(f.bit_rate)],
      ["Streams", f.nb_streams],
    ];
    for (const [k, v] of Object.entries(f.tags || {})) gen.push([`Tag ${k.replace(/_/g, " ")}`, v]);
    blocks.push(ffSection("General", gen));
  }
  const titleMap = { video: "Video", audio: "Audio", subtitle: "Text", data: "Data", attachment: "Attachment" };
  const counts = {}, seen = {};
  for (const st of (j.streams || [])) { const ty = st.codec_type || "data"; counts[ty] = (counts[ty] || 0) + 1; }
  for (const st of (j.streams || [])) {
    const ty = st.codec_type || "data";
    seen[ty] = (seen[ty] || 0) + 1;
    let title = titleMap[ty] || humanizeKey(ty);
    if (counts[ty] > 1) title += ` #${seen[ty]}`;
    const pairs = [], disp = [];
    for (const [k, v] of Object.entries(st)) {
      if (k === "index" || k === "codec_type" || k === "disposition") continue;
      if (k === "tags") { for (const [tk, tv] of Object.entries(v || {})) pairs.push([`Tag ${tk.replace(/_/g, " ")}`, tv]); continue; }
      pairs.push([humanizeKey(k), ffVal(k, v)]);
    }
    if (st.disposition) {
      for (const [dk, dv] of Object.entries(st.disposition)) if (dv) disp.push(dk);
      if (disp.length) pairs.push(["Disposition", disp.join(", ")]);
    }
    blocks.push(ffSection(title, pairs));
  }
  return blocks.filter(Boolean).join("\n\n");
}

// ======================= raw panels + search =======================
const panelEl = (name) => document.querySelector(`[data-panel="${name}"]`);
const tabEl = (name) => document.querySelector(`.tab[data-tab="${name}"]`);

function rawInfo(name) {
  const r = files[current]?.report;
  if (!r) return { ok: true, text: t("analyzing") };
  let tool = name === "mediainfo" ? (rawFull ? r.mediainfo_full : r.mediainfo) : r[name];
  if (!tool) return { ok: true, text: t("analyzing") };
  if (!tool.ok) return { ok: false, text: `⚠ ${tool.error}\n\n${tool.output || ""}` };
  let text = tool.output || t("empty");
  if (name === "exiftool" && !rawFull) text = compactExiftool(text);
  if (name === "ffprobe") text = formatFfprobe(tool.output) || text;
  return { ok: true, text };
}
function compactExiftool(text) {
  const noiseGroups = /^(ExifTool|System|File|QuickTime)$/i;
  const kept = text.split(/\r?\n/).filter((line) => {
    const group = line.match(/^\[([^\]]+)\]/)?.[1];
    return !group || !noiseGroups.test(group);
  });
  const compact = kept.join("\n").trim();
  return compact || t("exifCompactEmpty");
}
function setRaw(name) {
  const el = panelEl(name), tab = tabEl(name);
  const info = rawInfo(name);
  tab.classList.toggle("bad", !info.ok);
  applyHighlight(el, info.text);
}
function applyHighlight(el, text) {
  const term = searchTerm.trim();
  if (!term) { el.textContent = text; return; }
  const re = new RegExp(escRe(term), "gi");
  el.innerHTML = esc(text).replace(re, (m) => `<mark>${m}</mark>`);
}
function applySearch() {
  const countEl = document.getElementById("search-count");
  if (currentTab === "summary" || currentTab === "checks") { countEl.textContent = ""; return; }
  setRaw(currentTab);
  focusMatch(panelEl(currentTab).querySelectorAll("mark"));
}
function focusMatch(marks) {
  const countEl = document.getElementById("search-count");
  marks.forEach((m) => m.classList.remove("cur"));
  if (!searchTerm.trim()) { countEl.textContent = ""; return; }
  if (!marks.length) { countEl.textContent = "0/0"; return; }
  matchIdx = ((matchIdx % marks.length) + marks.length) % marks.length;
  const m = marks[matchIdx];
  m.classList.add("cur");
  m.scrollIntoView({ block: "center" });
  countEl.textContent = `${matchIdx + 1}/${marks.length}`;
}

// ======================= render =======================
function render() {
  const focusWorkspace = offloadMode || current < 0 || !files[current];
  document.getElementById("app").classList.toggle("focus-workspace", focusWorkspace);
  renderFileList();
  const empty = document.getElementById("empty");
  const viewer = document.getElementById("viewer");
  const cv = document.getElementById("compare-view");
  const bv = document.getElementById("batch-view");
  const ov = document.getElementById("offload-view");
  ov.hidden = !offloadMode;

  if (offloadMode) {
    empty.hidden = true; viewer.hidden = true; cv.hidden = true; bv.hidden = true;
    renderOffload();
    return;
  }
  if (batchMode && files.some((f) => batchCells(f))) {
    empty.hidden = true; viewer.hidden = true; cv.hidden = true; bv.hidden = false;
    renderBatch();
    return;
  }
  if (compareMode && files.length >= 2) {
    empty.hidden = true; viewer.hidden = true; cv.hidden = false; bv.hidden = true;
    renderCompare();
    return;
  }
  if (current < 0 || !files[current]) {
    empty.hidden = false; viewer.hidden = true; cv.hidden = true; bv.hidden = true;
    return;
  }
  empty.hidden = true; viewer.hidden = false; cv.hidden = true; bv.hidden = true;

  const f = files[current], r = f.report;
  const isDcp = f.kind === "dcp";
  document.getElementById("fhead-info").innerHTML =
    `<div class="fh-name">${esc(f.name)}</div><div class="fh-path">${esc(f.path)}</div>`;

  const simple = mode === "simple";

  // DCP-пакеты и Simple-режим не показывают технические и delivery-вкладки.
  ["mediainfo", "exiftool", "ffprobe"].forEach((n) => { tabEl(n).hidden = isDcp || simple; });
  tabEl("checks").hidden = isDcp || !!f.error || !r || simple;
  if ((isDcp || simple || !!f.error || !r) && currentTab === "checks") currentTab = "summary";
  if (isDcp || simple) currentTab = "summary";

  // summary
  const sumEl = panelEl("summary");
  if (f.error) sumEl.innerHTML = `<div class="err">${esc(f.error)}</div>`;
  else if (!r) sumEl.innerHTML = `<div class="load">${t("analyzing")}</div>`;
  else if (isDcp) sumEl.innerHTML = buildDcpHtml(f);
  else {
    const groups = buildSummary(f);
    let html = groups.map((grp) =>
      `<div class="card"><h3>${esc(t(grp.titleKey))}</h3><dl>` +
      grp.rows.map(([k, v]) => `<dt>${esc(t(k))}</dt><dd>${esc(String(v))}</dd>`).join("") +
      `</dl></div>`).join("");
    sumEl.innerHTML = html || `<div class="err">${t("noSummary")}</div>`;
  }

  const checksEl = panelEl("checks");
  if (!isDcp && !f.error && r && !simple) {
    checksEl.innerHTML = buildChecksHtml(f);
    fillSpecOptions();
    wireLoud(f);
    wireHashScan(f);
    document.getElementById("spec-run").onclick = () => {
      if (specProfile === "none") return;
      f.specRunProfile = specProfile;
      f.specResult = evalSpec(f, specProfile);
      render();
    };
    document.getElementById("spec-edit").onclick = openSpecEditor;
    document.getElementById("spec-profile").onchange = (e) => {
      specProfile = e.target.value;
      localStorage.setItem("specProfile", specProfile);
      logEvent(`spec profile -> ${specProfile}`);
      render();
    };
  } else {
    checksEl.innerHTML = "";
  }

  if (!isDcp) { setRaw("mediainfo"); setRaw("exiftool"); setRaw("ffprobe"); }

  // tab/panel visibility
  document.querySelectorAll(".panel").forEach((p) => { p.hidden = p.dataset.panel !== currentTab; });
  document.querySelectorAll(".tab").forEach((tb) => tb.classList.toggle("active", tb.dataset.tab === currentTab));
  document.getElementById("mi-mode").hidden = !["mediainfo", "exiftool"].includes(currentTab);
  document.getElementById("controls").hidden = currentTab === "summary" || currentTab === "checks";
  document.getElementById("mi-full").checked = rawFull;
  applySearch();
}

function renderFileList() {
  const listEl = document.getElementById("file-list");
  document.getElementById("file-count").textContent = files.length || "";
  const hasSetTools = files.length >= 2 && mode === "dit";
  const setActions = document.getElementById("set-actions");
  setActions.hidden = !hasSetTools;
  const batchButton = document.getElementById("batch-btn");
  const compareButton = document.getElementById("compare-btn");
  batchButton.hidden = !hasSetTools;
  compareButton.hidden = !hasSetTools || files.some((file) => file.kind === "dcp");
  listEl.innerHTML = "";
  files.forEach((f, i) => {
    const li = document.createElement("li");
    li.className = "file-item" + (i === current && !compareMode ? " active" : "");
    li.title = f.path;
    li.innerHTML = `<span class="fi-name">${esc(f.name)}</span><button class="fi-x" title="✕">×</button>`;
    li.addEventListener("click", (e) => {
      if (e.target.classList.contains("fi-x")) { removeFile(i); return; }
      current = i; compareMode = false; batchMode = false; offloadMode = false; render();
    });
    listEl.appendChild(li);
  });
}

// ======================= compare =======================
function flatSummary(f) {
  const map = new Map(), order = [];
  const groups = f.report && !f.error ? buildSummary(f) : [];
  for (const grp of groups)
    for (const [lbl, val] of grp.rows) {
      const key = grp.titleKey + "|" + lbl;
      if (!map.has(key)) order.push([grp.titleKey, lbl]);
      map.set(key, String(val));
    }
  return { map, order };
}
function renderCompare() {
  const cv = document.getElementById("compare-view");
  if (cmpA == null || cmpA >= files.length) cmpA = current >= 0 ? current : 0;
  if (cmpB == null || cmpB >= files.length || cmpB === cmpA)
    cmpB = files.findIndex((_, i) => i !== cmpA);

  const opts = (sel) => files.map((f, i) => `<option value="${i}" ${i === sel ? "selected" : ""}>${esc(f.name)}</option>`).join("");
  const fa = files[cmpA], fb = files[cmpB];
  const A = flatSummary(fa), B = flatSummary(fb);

  const groupOrder = ["grpMedia", "grpColor", "grpAudioProduction", "grpImage", "grpCamera"];
  let rows = "";
  for (const gk of groupOrder) {
    const labels = [];
    for (const [tk, lbl] of [...A.order, ...B.order])
      if (tk === gk && !labels.includes(lbl)) labels.push(lbl);
    if (!labels.length) continue;
    rows += `<tr class="grp"><td colspan="3">${esc(t(gk))}</td></tr>`;
    for (const lbl of labels) {
      const va = A.map.get(gk + "|" + lbl) || "—";
      const vb = B.map.get(gk + "|" + lbl) || "—";
      const diff = va !== vb ? " diff" : "";
      rows += `<tr class="cmp-row${diff}"><td class="cmp-lbl">${esc(t(lbl))}</td><td>${esc(va)}</td><td>${esc(vb)}</td></tr>`;
    }
  }

  cv.innerHTML =
    `<div class="cmp-head">
       <button id="cmp-close" type="button">${t("compareClose")}</button>
       <div class="cmp-pick">${t("comparePick")}
         <select id="cmp-a">${opts(cmpA)}</select>
         <select id="cmp-b">${opts(cmpB)}</select>
       </div>
     </div>
     <div class="cmp-table-wrap"><table class="cmp-table">
       <thead><tr><th>${t("colField")}</th><th>${esc(fa.name)}</th><th>${esc(fb.name)}</th></tr></thead>
       <tbody>${rows}</tbody>
     </table></div>`;

  document.getElementById("cmp-close").onclick = () => { compareMode = false; render(); };
  document.getElementById("cmp-a").onchange = (e) => { cmpA = +e.target.value; renderCompare(); };
  document.getElementById("cmp-b").onchange = (e) => { cmpB = +e.target.value; renderCompare(); };
}

// ======================= batch table (conform-readiness + delivery-set QC) =======================
// uniform=true — поле, которое в пачке сдачи должно совпадать (odd-one-out = баг).
// uniform=false — таймкод/reel, естественно разные по клипам (справка для конформа, без вороны).
const BATCH_COLS = [
  ["lblFps", "fps", true], ["lblResolution", "res", true], ["lblCodec", "codec", true],
  ["lblColor", "color", true], ["lblChannels", "channels", true], ["lblSampleRate", "srate", true],
  ["lblTimecode", "tc", false], ["lblReel", "reel", false],
];
function batchCells(f) {
  if (f.kind === "dcp" || f.error || !f.report) return null;
  const tr = miTracks(f.report);
  const v = trackOf(tr, "video"), a = trackOf(tr, "audio"), oth = trackOf(tr, "other");
  const ex = exifObj(f.report);
  const tc = (oth && oth.TimeCode_FirstFrame) || exifPickFirst(ex, ["TimeCode", "StartTimecode", "TimeReference"]);
  const reel = exifPickFirst(ex, ["Reel", "TapeName", "Tape"]);
  return {
    fps: v && v.FrameRate ? String(num(v.FrameRate)) : null,
    res: v && v.Width && v.Height ? `${v.Width}×${v.Height}` : null,
    codec: v ? v.Format || null : null,
    color: v ? v.ColorSpace || v.colour_primaries || null : null,
    channels: a && a.Channels != null ? String(a.Channels) : null,
    srate: a && a.SamplingRate ? `${(num(a.SamplingRate) / 1000).toFixed(1)}k` : null,
    tc: tc || null,
    reel: reel || null,
    verdict: f.specRunProfile === specProfile ? f.specResult?.verdict || null : null,
  };
}
function renderBatch() {
  const bv = document.getElementById("batch-view");
  const rows = files.map((f) => ({ f, c: batchCells(f) })).filter((x) => x.c);
  const showVerdict = rows.some(({ c }) => c.verdict != null);
  const head =
    `<div class="cmp-head">
       <button id="batch-close" type="button">${t("batchClose")}</button>
       <div class="cmp-pick">${esc(t("batchTitle"))} · ${rows.length}</div>
     </div>`;
  const closeWire = () => { document.getElementById("batch-close").onclick = () => { batchMode = false; render(); }; };
  if (!rows.length) { bv.innerHTML = head + `<div class="load" style="padding:20px">${t("batchNoData")}</div>`; closeWire(); return; }

  // «Белая ворона»: для uniform-полей находим мажоритарное значение, меньшинство подсвечиваем.
  const odd = {};
  for (const [, key, uniform] of BATCH_COLS) {
    if (!uniform) continue;
    const freq = new Map();
    for (const { c } of rows) { const val = c[key]; if (val == null) continue; freq.set(val, (freq.get(val) || 0) + 1); }
    if (freq.size < 2) continue; // всё одинаково (или одно значение) — вороны нет
    const maxN = Math.max(...freq.values());
    odd[key] = new Set([...freq.entries()].filter(([, n]) => n < maxN).map(([v]) => v));
  }

  const vdot = { pass: "b-pass", warn: "b-warn", fail: "b-fail" };
  const th = `<th>${t("colField")}</th>` +
    (showVerdict ? `<th>${esc(t("batchVerdict"))}</th>` : "") +
    BATCH_COLS.map(([lbl]) => `<th>${esc(t(lbl))}</th>`).join("");
  const body = rows.map(({ f, c }) => {
    const cells = BATCH_COLS.map(([, key]) => {
      const val = c[key];
      const cls = (odd[key] && val != null && odd[key].has(val)) ? ' class="b-odd"' : "";
      return `<td${cls}>${esc(val == null ? "—" : val)}</td>`;
    }).join("");
    const vcell = showVerdict
      ? `<td>${c.verdict ? `<span class="b-dot ${vdot[c.verdict]}" title="${c.verdict}"></span>` : "—"}</td>` : "";
    return `<tr><td class="cmp-lbl" title="${esc(f.path)}">${esc(f.name)}</td>${vcell}${cells}</tr>`;
  }).join("");

  bv.innerHTML = head +
    `<div class="cmp-table-wrap"><table class="cmp-table batch-table">
       <thead><tr>${th}</tr></thead><tbody>${body}</tbody>
     </table>
     <div class="loud-hint" style="padding:10px 4px">${esc(t("batchHint"))}</div></div>`;
  closeWire();
}

// ======================= loading =======================
const bad = (e) => ({ ok: false, output: "", error: typeof e === "string" ? e : "no data" });
async function addPaths(paths) {
  if (!paths.length) return;
  // Inspecting media is a change of workspace, not a destructive reset: the
  // configured offload source/destinations stay in `off` for a later return.
  offloadMode = false;
  ensureWorkspaceWindow("inspect");
  for (const path of paths) {
    if (files.some((f) => f.path === path)) continue;
    const name = path.split(/[\\/]/).pop();
    let isDir = false;
    try { isDir = await invoke("is_directory", { path }); } catch {}
    const entry = { id: ++seq, path, name, kind: isDir ? "dcp" : "file", report: null, error: null };
    files.push(entry);
    current = files.length - 1;
    compareMode = false;
    render();
    try {
      entry.report = await invoke(isDir ? "analyze_dcp" : "analyze_file", { path });
    } catch (e) {
      entry.error = typeof e === "string" ? e : (e?.message || "analyze error");
      if (!isDir)
        entry.report = { size_bytes: 0, summary_mi_json: "", summary_exif_json: "", mediainfo: bad(e), mediainfo_full: bad(e), exiftool: bad(e), ffprobe: bad(e) };
    }
    render();
  }
}
function removeFile(i) {
  files.splice(i, 1);
  if (current >= files.length) current = files.length - 1;
  cmpA = cmpB = null;
  if (files.length < 2) { compareMode = false; batchMode = false; }
  render();
}

// ======================= report =======================
function buildMarkdown(f) {
  if (f.kind === "dcp") return buildDcpMarkdown(f);
  const r = f.report;
  const groups = f.error ? [] : buildSummary(f);
  const now = new Date().toISOString().replace("T", " ").slice(0, 19);
  let md = `# ProofCat — ${f.name}\n\n`;
  md += `- **File:** \`${f.path}\`\n`;
  md += `- **Size:** ${fmtSize(r.size_bytes)} (${r.size_bytes} B)\n`;
  md += `- **Analyzed:** ${now}\n`;
  md += `- **Tools:** MediaInfo · ExifTool · FFprobe\n\n`;
  if (groups.length) {
    md += `## Summary\n\n| Section | Field | Value |\n|---|---|---|\n`;
    for (const grp of groups)
      for (const [k, v] of grp.rows)
        md += `| ${t(grp.titleKey, "en")} | ${t(k, "en")} | ${String(v).replace(/\|/g, "\\|")} |\n`;
    md += `\n`;
  }
  const L = f.loud;
  if (L && L.ok) {
    md += `## Loudness (EBU R128)\n\n`;
    if (L.integrated != null) md += `- **Integrated:** ${L.integrated.toFixed(1)} LUFS\n`;
    if (L.true_peak != null) md += `- **True Peak:** ${L.true_peak.toFixed(1)} dBTP\n`;
    if (L.lra != null) md += `- **Range (LRA):** ${L.lra.toFixed(1)} LU\n`;
    md += `- _Targets: YouTube -14 · Broadcast -23 LUFS · Peak <= -1 dBTP_\n\n`;
  }
  if (f.specRunProfile === specProfile && !f.error) {
    const res = f.specResult || f.specEval || evalSpec(f, specProfile);
    md += `## Delivery check — ${res.profile.label.en}\n\n- **Verdict:** ${res.verdict.toUpperCase()}\n\n`;
    const tag = { ok: "OK", info: "i", warn: "WARN", err: "FAIL" };
    const ordr = { err: 0, warn: 1, info: 2, ok: 3 };
    res.checks.slice().sort((x, y) => ordr[x.level] - ordr[y.level]).forEach((c) => { md += `- [${tag[c.level]}] (${c.scope}) ${c.text}\n`; });
    md += `\n`;
  }
  if (f.hash && f.hash.ok) md += `## Checksum\n\n- **SHA-256:** \`${f.hash.hash}\`\n\n`;
  if (f.scan && f.scan.ok) {
    md += `## Frame scan (black / frozen)\n\n`;
    const segs = f.scan.segments || [];
    if (!segs.length) md += `- No black or frozen frames found\n\n`;
    else {
      segs.forEach((s) => {
        const k = s.kind === "black" ? "Black" : "Frozen";
        const rng = s.end != null ? `${fmtDur(s.start)}–${fmtDur(s.end)}` : `${fmtDur(s.start)}→`;
        md += `- [${k}] ${rng}\n`;
      });
      md += `\n`;
    }
  }
  const block = (title, tool) => {
    md += `## ${title}\n\n`;
    if (!tool || !tool.ok) md += `> ⚠ ${tool ? tool.error : "no data"}\n\n`;
    md += "```\n" + ((tool && tool.output) || "") + "\n```\n\n";
  };
  block("MediaInfo (-f)", r.mediainfo_full || r.mediainfo);
  block("ExifTool", r.exiftool);
  const ffText = r.ffprobe && r.ffprobe.ok ? (formatFfprobe(r.ffprobe.output) || r.ffprobe.output) : null;
  block("FFprobe", r.ffprobe ? { ok: r.ffprobe.ok, output: ffText, error: r.ffprobe.error } : null);
  return md;
}

async function copyText(text) {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch {
    const ta = document.createElement("textarea");
    ta.value = text; ta.style.position = "fixed"; ta.style.opacity = "0";
    document.body.appendChild(ta); ta.select();
    let ok = false;
    try { ok = document.execCommand("copy"); } catch {}
    document.body.removeChild(ta);
    return ok;
  }
}

// ======================= language apply =======================
function applyLang() {
  document.documentElement.lang = lang;
  document.querySelectorAll("[data-i18n]").forEach((el) => { el.textContent = t(el.dataset.i18n); });
  document.getElementById("save-btn").textContent = t("save");
  document.getElementById("copy-btn").textContent = t("copy");
  document.getElementById("search").placeholder = t("searchPlaceholder");
  document.querySelector("#feedback-btn .btn-label").textContent = t("feedbackBtn");
  document.querySelector("#logs-btn .btn-label").textContent = t("logsBtn");
  document.querySelectorAll("[data-lang]").forEach((b) => b.classList.toggle("active", b.dataset.lang === lang));
  applyModeToggle();
  setUpdateBtn();
  fillSpecOptions();
  render();
}

function applyModeToggle() {
  document.getElementById("mode-toggle").title = t("modeTitle");
  document.querySelectorAll("[data-mode]").forEach((b) => b.classList.toggle("active", b.dataset.mode === mode));
}

// ======================= events =======================
document.querySelectorAll(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    currentTab = tab.dataset.tab;
    matchIdx = 0;
    document.querySelectorAll(".tab").forEach((tb) => tb.classList.remove("active"));
    tab.classList.add("active");
    document.querySelectorAll(".panel").forEach((p) => { p.hidden = p.dataset.panel !== currentTab; });
    document.getElementById("mi-mode").hidden = !["mediainfo", "exiftool"].includes(currentTab);
    document.getElementById("controls").hidden = currentTab === "summary" || currentTab === "checks";
    applySearch();
  });
});
document.getElementById("mi-full").addEventListener("change", (e) => {
  rawFull = e.target.checked; matchIdx = 0; setRaw(currentTab); applySearch();
});
const searchEl = document.getElementById("search");
searchEl.addEventListener("input", () => { searchTerm = searchEl.value; matchIdx = 0; applySearch(); });
searchEl.addEventListener("keydown", (e) => {
  if (e.key === "Enter") { e.preventDefault(); matchIdx += e.shiftKey ? -1 : 1; focusMatch(panelEl(currentTab).querySelectorAll("mark")); }
});
window.addEventListener("keydown", (e) => {
  if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
    if (!searchEl.offsetParent) return;
    e.preventDefault(); searchEl.focus(); searchEl.select();
  }
});
document.getElementById("save-btn").addEventListener("click", async () => {
  const f = files[current];
  if (!f || !f.report) return;
  const status = document.getElementById("save-status");
  const base = f.name.replace(/\.[^.]+$/, "");
  try {
    const saved = await invoke("save_report", { defaultName: `${base}.meta.md`, contents: buildMarkdown(f) });
    status.textContent = saved ? `✓ ${t("savedPrefix")}${saved}` : "";
  } catch (e) {
    status.textContent = `⚠ ${typeof e === "string" ? e : "save error"}`;
  }
});
document.getElementById("copy-btn").addEventListener("click", async () => {
  const f = files[current];
  if (!f || !f.report) return;
  const status = document.getElementById("save-status");
  status.textContent = (await copyText(buildMarkdown(f))) ? t("copied") : "⚠";
  logEvent("report copied");
});
document.getElementById("add-btn").addEventListener("click", async () => {
  const paths = await invoke("pick_files");
  if (paths && paths.length) addPaths(paths);
});
document.getElementById("empty-inspect-btn").addEventListener("click", () => {
  document.getElementById("add-btn").click();
});
document.getElementById("empty-offload-btn").addEventListener("click", () => {
  document.getElementById("offload-btn").click();
});
document.getElementById("compare-btn").addEventListener("click", () => {
  if (files.length < 2) return;
  compareMode = true; cmpA = current; cmpB = null; render();
  logEvent("compare opened");
});
document.getElementById("batch-btn").addEventListener("click", () => {
  if (files.length < 2) return;
  batchMode = true; compareMode = false; render();
  logEvent("batch opened");
});

/* ── Copy mode: один источник на N назначений с verify + ASC MHL ─────────── */

// Экран вердикта. Единственное решение, ради которого существует продукт:
// можно ли стирать карту. Формулируем крупно и однозначно.
const VERDICT_ICONS = {
  safe: '<circle cx="12" cy="12" r="10"/><path d="M8 12.5l2.6 2.6L16 9"/>',
  fail: '<circle cx="12" cy="12" r="10"/><path d="M15 9l-6 6M9 9l6 6"/>',
  archive: '<path d="M4 7h16v13H4zM4 7l1.5-3h13L20 7M9 11h6"/>',
  copy: '<rect x="8" y="8" width="12" height="12" rx="2"/><path d="M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3"/>',
};

function verdictHeroHtml(s) {
  const map = {
    SAFE_TO_FORMAT: { tone: "safe", icon: "safe", word: "vSafeWord", sub: "vSafeSub" },
    ARCHIVE_VERIFIED: { tone: "neutral", icon: "archive", word: "vArchiveWord", sub: "vArchiveSub" },
    COPY_COMPLETE: { tone: "neutral", icon: "copy", word: "vCopyWord", sub: "vCopySub" },
    FAILED: { tone: "fail", icon: "fail", word: "vFailedWord", sub: "vFailedSub" },
  };
  const v = map[s.verdict] || map.FAILED;
  return (
    `<div class="verdict verdict-${v.tone}">
       <svg class="verdict-icon" viewBox="0 0 24 24" fill="none" stroke-width="2.3" stroke-linecap="round" stroke-linejoin="round">${VERDICT_ICONS[v.icon]}</svg>
       <div class="verdict-word">${esc(t(v.word))}</div>
       <div class="verdict-sub">${esc(t(v.sub))}</div>
     </div>`
  );
}

// Улики строятся ТОЛЬКО из того, что реально вернул движок (summary.replicas).
// Ничего не додумываем: если назначение не перечитано — так и пишем.
function evidenceHtml(s) {
  const algos = (s.hashPolicy?.evidenceAlgorithms || []).map((a) => String(a).toUpperCase()).join(" + ");
  const byDest = new Map();
  (s.replicas || []).forEach((r) => {
    const cur = byDest.get(r.destination) || { total: 0, verified: 0, failed: 0 };
    cur.total += 1;
    if (r.status === "verified" || r.status === "alreadyMatched") cur.verified += 1;
    if (r.status === "copyFailed" || r.status === "verifyFailed" || r.status === "sourceChanged") cur.failed += 1;
    byDest.set(r.destination, cur);
  });

  const rows = [...byDest.entries()].map(([dest, c]) => {
    const ok = c.failed === 0 && c.verified === c.total;
    return (
      `<div class="off-ev-row">
         <span class="off-ev-k">${esc(dest.split("/").pop() || dest)}</span>
         <span class="off-ev-v">${c.verified}/${c.total} · ${esc(algos || "—")}</span>
         <span class="off-ev-s ${ok ? "off-ev-ok" : "off-ev-bad"}">${ok ? esc(t("offReadbackOk")) : esc(t("offReadbackFail"))}</span>
       </div>`
    );
  });

  if (!rows) return "";
  const shownRows = off.evidenceExpanded ? rows : rows.slice(0, 3);
  const toggle = rows.length > 3
    ? `<button id="off-evidence-toggle" class="off-evidence-more" type="button">${esc(t(off.evidenceExpanded ? "offHideEvidence" : "offShowEvidence"))}</button>`
    : "";
  return (
    `<div class="off-replicas">
       <div class="off-replicas-n"><b>${s.verifiedReplicas ?? 0}</b><span>${esc(t("offFileCopies"))}</span></div>
       <div class="off-algos">${esc(algos)}</div>
     </div>
     <div class="off-evidence">${shownRows.join("")}${toggle}</div>`
  );
}

function renderOffload() {
  const ov = document.getElementById("offload-view");
  const canVerifyExisting = !off.running && !off.summary;
  const head =
    `<div class="off-head">
       <button id="off-close" type="button">${t("offloadClose")}</button>
       <div class="off-head-title">${esc(t("offloadTitle"))}</div>
       ${canVerifyExisting
         ? `<button id="off-verify-folder" class="off-verify-existing" type="button">${t("offVerifyFolder")}</button>`
         : `<span aria-hidden="true"></span>`}
     </div>`;
  let body = "";

  if (off.running) {
    const p = off.prog || {};
    const taskProgress = offloadTaskProgress(p);
    const phaseKey = { scanning: "offPhaseScanning", sourcePreRead: "offPhaseSourcePreRead", copying: "offPhaseCheckingExisting", copyingData: "offPhaseCopying", destinationVerify: "offPhaseDestinationVerify", repairing: "offPhaseRepairing", repairingData: "offPhaseRepairing", repairReadback: "offPhaseRepairing", manualVerify: "offPhaseManualVerify", mhl: "offPhaseMhl", done: "offPhaseDone" }[p.phase] || "offPhaseScanning";
    const currentFileLabel = taskProgress.total
      ? template("offCurrentFile", { current: taskProgress.current, total: taskProgress.total })
      : "";
    const speed = off.transferSpeedBps > 0 ? `${t("offTransferSpeed")}: ${fmtTransferSpeed(off.transferSpeedBps)}` : "";
    const isWritingFile = ["copyingData", "repairingData"].includes(p.phase) && Number(p.bytesTotal) > 0;
    const filePercent = isWritingFile
      ? Math.round(Math.min(1, Math.max(0, Number(p.bytesDone || 0) / Number(p.bytesTotal))) * 100)
      : 0;
    const fileProgress = isWritingFile
      ? `<div class="off-current-progress"><div class="off-progress-heading"><div class="off-current-label">${esc(currentFileLabel)}</div><div class="off-progress-percent"><b>${filePercent}%</b></div></div><div class="off-bar off-current-bar" aria-label="${esc(`${currentFileLabel}: ${filePercent}%`)}"><div class="off-bar-fill" style="width:${filePercent}%"></div></div><div class="off-progress-detail"><span>${fmtSize(p.bytesDone || 0)} / ${fmtSize(p.bytesTotal || 0)}</span>${speed ? `<span>${esc(speed)}</span>` : ""}</div></div>`
      : `<div class="off-progress-detail"><span>${esc(currentFileLabel)}</span></div>`;
    body =
      `<div class="off-body">
         <div class="off-progress-heading"><div class="off-phase">${t(phaseKey)}${off.paused ? ` · ${t("offPausedTag")}` : ""}</div><div class="off-progress-percent">${t("offOverallProgress")}: <b>${taskProgress.percent}%</b></div></div>
         <div class="off-file">${esc(p.currentFile || "")}</div>
         <div class="off-bar" aria-label="${esc(`${t("offOverallProgress")}: ${taskProgress.percent}%`)}"><div class="off-bar-fill" style="width:${taskProgress.percent}%"></div></div>
         ${fileProgress}
         <button id="off-pause" class="set-btn" type="button">${t(off.paused ? "offResume" : "offPause")}</button>
         <button id="off-cancel" class="set-btn" type="button">${t("offCancel")}</button>
       </div>`;
  } else if (off.summary) {
    const s = off.summary;
    const isSafe = s.verdict === "SAFE_TO_FORMAT";
    const errRows = s.failures.map((x) => `<li>✗ ${esc(x.file)} — ${esc(localizeOffloadMessage(x.error))}</li>`).join("");
    const mhlLocations = [...new Map((s.mhlPaths || []).map((mhlPath) => {
      const mhlFolder = mhlPath.replace(/[\\/][^\\/]+$/, "");
      const destination = mhlFolder.replace(/[\\/]ascmhl$/i, "");
      return [destination, { destination, mhlPath }];
    })).values()];
    const mhlControl = mhlLocations.length
      ? `<button class="set-btn off-show-mhl" data-mhl-paths="${esc(JSON.stringify(mhlLocations.map(({ mhlPath }) => mhlPath)))}" type="button">${esc(t("offShowMhl"))}</button>`
      : "";
    const warningRows = (s.warnings || []).map((x) => `<li>⚠ ${esc(localizeOffloadMessage(x))}</li>`).join("");
    const verifyWasCancelled = isCancelledVerification(off.verifyError);
    const verifyResult = off.verifyReport
      ? `<div class="card dcp-verdict ${off.verifyReport.summary.success ? "dcp-pass" : "dcp-fail"} off-reverify-result"><h3>${off.verifyReport.summary.success ? "✓" : "✗"} ${t(off.verifyReport.summary.success ? "offVerifyOk" : "offVerifyFail")}</h3><div class="off-hint">${off.verifyReport.summary.passed} ${t("offVerifyPassed")} · ${off.verifyReport.summary.failed} ${t("offVerifyFailed")} · ${off.verifyReport.summary.missing} ${t("offVerifyMissing")}</div></div>`
      : off.verifyError
        ? `<div class="card dcp-verdict ${verifyWasCancelled ? "dcp-warn" : "dcp-fail"} off-reverify-result"><h3>${verifyWasCancelled ? "•" : "✗"} ${esc(t(verifyWasCancelled ? "offVerifyCancelled" : "offVerifyFail"))}</h3><div class="off-hint">${esc(localizeOffloadMessage(off.verifyError))}</div></div>`
        : "";
    body =
      `<div class="off-body">
         ${verdictHeroHtml(s)}
         ${evidenceHtml(s)}
         <div class="off-summary-stats">${t("offCopied")}: <b>${s.copied}</b> · ${t("offSkipped")}: <b>${s.skipped}</b> · ${t("offFailed")}: <b>${s.failed}</b> · ${t("offBytes")}: <b>${fmtSize(s.bytesCopied)}</b></div>
         ${s.jobId ? `<div class="off-result-verify"><button id="off-reverify" class="set-btn" type="button">${esc(t("offReverify"))}</button><div class="off-hint">${esc(t("offReverifyHint"))}</div>${verifyResult}</div>` : ""}
         ${isSafe
           ? `<div class="off-action off-action-safe"><div><div class="off-action-title">${esc(t("offSafeActionTitle"))}</div><div class="off-action-body">${esc(t("offSafeActionBody"))}</div></div></div>`
           : warningRows ? `<details class="off-warning-details"><summary>${esc(t("offConditions"))}</summary><ul class="off-action-reasons">${warningRows}</ul><div class="off-action-body">${esc(t("offNotSafeNote"))}</div></details>` : ""}
         ${errRows ? `<div class="off-lbl">${t("offErrList")}</div><ul class="dcp-checks">${errRows}</ul>` : ""}
         <div class="off-result-tools${mhlControl ? "" : " off-result-tools-single"}">
           ${mhlControl ? `<div class="off-mhl-locations"><div class="off-lbl">${esc(t("offMhlOut"))}</div>${mhlControl}</div>` : ""}
           <div class="off-report-export"><div class="off-lbl">${t("offExport")}</div><div class="off-export-row"><button class="set-btn off-export" data-format="json" type="button">JSON</button><button class="set-btn off-export" data-format="html" type="button">HTML</button><button class="set-btn off-export" data-format="csv" type="button">CSV</button><button class="set-btn off-export" data-format="txt" type="button">TXT</button></div></div>
         </div>
         <button id="off-clear" class="loud-btn" type="button" style="margin-top:16px">${t("offClear")}</button>
       </div>`;
  } else {
    const isCancelled = /cancel|отмен/i.test(off.error);
    const destRows = off.dests.map((d, i) =>
      `<div class="off-card">
         <div class="off-card-icon">${i + 1}</div>
         <div class="off-card-body">
           <div class="off-card-name">${esc(pathBasename(d))}</div>
           <div class="off-card-path">${esc(d)}</div>
         </div>
         <button class="off-del" data-di="${i}" type="button">✕</button>
       </div>`).join("");
    const diskSvg = `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><rect x="4" y="3" width="16" height="18" rx="2"></rect><path d="M9 3v5h6V3" stroke-linecap="round"></path></svg>`;
    const srcCard = off.source
      ? `<div class="off-card">
           <div class="off-card-icon lg">${diskSvg}</div>
           <div class="off-card-body">
             <div class="off-card-name">${esc(pathBasename(off.source))}</div>
             <div class="off-card-path">${esc(off.source)}</div>
           </div>
           <button id="off-src" class="set-btn" type="button">${t("offChange")}</button>
         </div>`
      : `<button id="off-src" class="off-add-dashed" type="button">${diskSvg}${t("offPick")}</button>`;
    const profilePicker = mode === "dit"
      ? `<div class="off-lbl">${t("offProfile")}</div>
         <div class="off-profiles">
           <button type="button" class="off-profile-card${off.profile === "archiveMax" ? " active" : ""}" data-profile="archiveMax">
             <div class="off-profile-name">${t("amName")}<span class="off-profile-badge">${t("recommended")}</span></div>
             <div class="off-profile-tag">${t("amTag")}</div>
             <div class="off-profile-desc">${t("amDesc")}</div>
           </button>
           <button type="button" class="off-profile-card${off.profile === "fast" ? " active" : ""}" data-profile="fast">
             <div class="off-profile-name">${t("fastName")}</div>
             <div class="off-profile-tag">${t("fastTag")}</div>
             <div class="off-profile-desc">${t("fastDesc")}</div>
           </button>
         </div>`
      : "";
    body =
      `<div class="off-body">
         ${off.error ? `<div class="card dcp-verdict dcp-fail"><h3>✗ ${esc(isCancelled ? t("offCancelledTitle") : off.error)}</h3></div>` : ""}
         <div class="off-lbl">${t("offSource")}</div>
         ${srcCard}
         <div class="off-lbl">${t("offDests")}</div>
         ${destRows}
         <button id="off-add" class="off-add-dashed" type="button"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 5v14M5 12h14"/></svg>${t("offAddDest")}</button>
         ${off.verifyReport ? `<div class="card dcp-verdict ${off.verifyReport.summary.success ? "dcp-pass" : "dcp-fail"}"><h3>${off.verifyReport.summary.success ? "✓" : "✗"} ${t(off.verifyReport.summary.success ? "offVerifyOk" : "offVerifyFail")}</h3><div class="off-hint">${off.verifyReport.summary.passed} ${t("offVerifyPassed")} · ${off.verifyReport.summary.failed} ${t("offVerifyFailed")} · ${off.verifyReport.summary.missing} ${t("offVerifyMissing")}</div></div>` : ""}
         ${profilePicker}
         <div class="off-mhl-note"><strong>${t("offMhlInfo")}</strong><span>${t("offMhlRequired")}</span></div>
         <button id="off-advanced-toggle" type="button" aria-expanded="${off.advancedOpen}" class="off-advanced-toggle${off.advancedOpen ? " open" : ""}"><span class="off-advanced-copy"><span>${t("offAdvanced")}</span><small>${t("offAdvancedHint")}</small></span><span class="chev">▾</span></button>
         <div class="off-advanced-body"${off.advancedOpen ? "" : " hidden"}>
           <div class="off-lbl">${t("offExtraHashes")}</div>
           <div class="off-hint">${t("offExtraHashesHint")}</div>
           <label class="off-switch-row"><span>SHA-256</span><span class="switch"><input class="off-extra" value="sha256" type="checkbox"${off.extras.includes("sha256") ? " checked" : ""}/><span class="switch-track"></span></span></label>
           <label class="off-switch-row"><span>MD5</span><span class="switch"><input class="off-extra" value="md5" type="checkbox"${off.extras.includes("md5") ? " checked" : ""}/><span class="switch-track"></span></span></label>
           <div class="off-lbl">${t("offContacts")}</div>
           <textarea id="off-contacts" rows="3" class="spec-input" placeholder="Name | DIT | phone or email">${esc(contactsToText(off.contacts))}</textarea>
           <div class="off-hint">${t("offContactsHint")}</div>
           <label class="off-switch-row"><span>${t("offNotifyDone")}</span><span class="switch"><input id="off-notify" type="checkbox"${off.notifyWhenDone ? " checked" : ""}/><span class="switch-track"></span></span></label>
         </div>
         <button id="off-start" class="loud-btn off-start" type="button"${off.source && off.dests.length ? "" : " disabled"}><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"><path d="M6 4l14 8-14 8V4z"/></svg>${t("offStart")}</button>
         ${localStorage.getItem("lastOffloadJob") ? `<button id="off-resume-job" class="set-btn" type="button">Resume ${esc(localStorage.getItem("lastOffloadJob"))}</button>` : ""}
         <div class="off-hint">${t("offSrcNote")} ${t("offCherryPickWarning")}</div>
       </div>`;
  }

  ov.innerHTML = head + body;

  document.getElementById("off-close").onclick = () => { offloadMode = false; render(); };
  const byId = (id) => document.getElementById(id);
  if (off.running) {
    byId("off-pause").onclick = () => {
      off.paused = !off.paused;
      invoke("offload_set_pause", { paused: off.paused });
      renderOffload();
    };
    byId("off-cancel").onclick = () => { off.paused = false; invoke("offload_cancel"); };
  } else if (off.summary) {
    ov.querySelectorAll(".off-export").forEach((button) => {
      button.onclick = async () => {
        try {
          await invoke("offload_export", { jobId: off.summary.jobId, format: button.dataset.format });
        } catch (e) {
          off.error = String(e); renderOffload();
        }
      };
    });
    ov.querySelectorAll(".off-show-mhl").forEach((button) => {
      button.onclick = async () => {
        try {
          await invoke("offload_open_mhl_folders", { mhlPaths: JSON.parse(button.dataset.mhlPaths || "[]") });
        } catch (e) {
          console.error("Unable to open MHL folder", e);
        }
      };
    });
    byId("off-clear").onclick = () => { offResetFresh(); renderOffload(); };
    const evidenceToggle = byId("off-evidence-toggle");
    if (evidenceToggle) evidenceToggle.onclick = () => { off.evidenceExpanded = !off.evidenceExpanded; renderOffload(); };
    const reverifyButton = byId("off-reverify");
    if (reverifyButton) reverifyButton.onclick = reverifyCurrentOffload;
  } else {
    byId("off-src").onclick = async () => {
      const p = await invoke("offload_pick_folder");
      if (p) { off.source = p; off.error = ""; renderOffload(); }
    };
    byId("off-add").onclick = async () => {
      const p = await invoke("offload_pick_folder");
      if (p && !off.dests.includes(p)) { off.dests.push(p); off.error = ""; renderOffload(); }
    };
    ov.querySelectorAll(".off-del").forEach((b) => {
      b.onclick = () => { off.dests.splice(Number(b.dataset.di), 1); renderOffload(); };
    });
    ov.querySelectorAll(".off-profile-card").forEach((b) => {
      b.onclick = () => { off.profile = b.dataset.profile; renderOffload(); };
    });
    if (byId("off-advanced-toggle")) byId("off-advanced-toggle").onclick = () => {
      off.advancedOpen = !off.advancedOpen; renderOffload();
    };
    ov.querySelectorAll(".off-extra").forEach((checkbox) => {
      checkbox.onchange = () => {
        off.extras = [...ov.querySelectorAll(".off-extra:checked")].map((item) => item.value);
      };
    });
    byId("off-contacts").onchange = (e) => {
      off.contacts = parseContacts(e.target.value);
      localStorage.setItem("offloadContacts", JSON.stringify(off.contacts));
    };
    byId("off-notify").onchange = (e) => {
      off.notifyWhenDone = e.target.checked;
      localStorage.setItem("offloadNotifyWhenDone", String(off.notifyWhenDone));
    };
    byId("off-start").onclick = startOffload;
    const resumeButton = byId("off-resume-job");
    if (resumeButton) resumeButton.onclick = async () => {
      off.running = true; off.error = ""; off.prog = null; resetOffloadSpeed(); renderOffload();
      try {
        off.summary = await invoke("offload_resume", { jobId: localStorage.getItem("lastOffloadJob") });
      } catch (e) {
        off.error = String(e);
      }
      off.running = false; renderOffload();
    };
    byId("off-verify-folder").onclick = async () => {
      const path = await invoke("offload_pick_folder");
      if (!path) return;
      off.running = true; off.paused = false; off.prog = null; renderOffload();
      try {
        off.verifyReport = await invoke("offload_verify", { path, jobId: null, options: { verifyAllGenerations: true } });
        off.error = "";
      } catch (e) {
        off.verifyReport = null; off.error = String(e);
      }
      off.running = false;
      renderOffload();
    };
  }
}

async function reverifyCurrentOffload() {
  if (off.running || !off.summary?.jobId) return;
  off.running = true;
  off.paused = false;
  resetOffloadSpeed();
  off.verifyError = "";
  off.prog = { phase: "manualVerify", currentFile: "", fileIndex: 0, totalFiles: 0, bytesDone: 0, bytesTotal: 0 };
  renderOffload();
  try {
    off.verifyReport = await invoke("offload_verify", {
      path: null,
      jobId: off.summary.jobId,
      options: { verifyAllGenerations: true },
    });
  } catch (e) {
    off.verifyReport = null;
    off.verifyError = String(e);
  }
  off.running = false;
  off.paused = false;
  if (offloadMode) renderOffload();
}

async function startOffload() {
  if (off.running || !off.source || !off.dests.length) return;
  off.running = true; off.paused = false; off.summary = null; off.error = ""; off.prog = null; resetOffloadSpeed();
  renderOffload();
  logEvent("offload started");
  if (off.notifyWhenDone && "Notification" in window && Notification.permission === "default") {
    Notification.requestPermission().catch(() => {});
  }
  try {
    off.jobId = await invoke("offload_start", {
      source: off.source,
      destinations: off.dests,
      algorithms: ["xxh64", ...(off.profile === "archiveMax" ? ["blake3"] : []), ...off.extras],
      writeMhl: true,
      profile: off.profile,
      smallFileConcurrency: 1,
      reportContacts: off.contacts,
      autoEject: false,
    });
    localStorage.setItem("lastOffloadJob", off.jobId);
    logEvent(`offload started: ${off.jobId}`);
  } catch (e) {
    off.error = String(e);
    logEvent(`offload error: ${off.error}`);
    off.running = false;
  }
  if (offloadMode) renderOffload();
}

listen("offload-progress", (e) => {
  off.prog = e.payload;
  updateOffloadTransferSpeed(e.payload);
  if (offloadMode && off.running) renderOffload();
});
listen("offload-complete", (e) => {
  off.summary = e.payload; off.running = false; off.paused = false; resetOffloadSpeed();
  logEvent(`offload done: ${off.summary.copied} copied, ${off.summary.failed} failed`);
  if (off.notifyWhenDone && "Notification" in window && Notification.permission === "granted") {
    const title = "ProofCat";
    const body = off.summary.verdict === "SAFE_TO_FORMAT"
      ? t("offSafeTitle")
      : `${off.summary.verdict}: ${off.summary.failed || 0} ${t("offFailed").toLowerCase()}`;
    try { new Notification(title, { body }); } catch (_) { /* completion remains visible in the app */ }
  }
  if (offloadMode) renderOffload();
});
listen("offload-error", (e) => {
  off.error = e.payload?.message || String(e.payload); off.running = false; off.paused = false; resetOffloadSpeed();
  if (e.payload?.jobId) localStorage.setItem("lastOffloadJob", e.payload.jobId);
  logEvent(`offload error: ${off.error}`);
  if (offloadMode) renderOffload();
});

document.getElementById("offload-btn").addEventListener("click", () => {
  // This is navigation, not a reset: returning from Inspect must retain the
  // selected source, destinations and options. A completed transfer has its
  // own explicit “New offload” action.
  ensureWorkspaceWindow("offload");
  offloadMode = true; batchMode = false; compareMode = false; render();
  logEvent("offload opened");
});
document.querySelectorAll("[data-lang]").forEach((b) => {
  b.addEventListener("click", () => {
    lang = b.dataset.lang; localStorage.setItem("lang", lang); applyLang();
    logEvent(`lang -> ${lang}`);
  });
});
document.querySelectorAll("[data-theme-set]").forEach((b) => {
  b.addEventListener("click", () => {
    theme = b.dataset.themeSet; localStorage.setItem("theme", theme); applyTheme();
    logEvent(`theme -> ${theme}`);
  });
});
document.querySelectorAll("[data-mode]").forEach((b) => {
  b.addEventListener("click", () => {
    mode = b.dataset.mode; localStorage.setItem("mode", mode);
    if (mode === "simple") {
      off.profile = "archiveMax";
      if (batchMode || compareMode) { batchMode = false; compareMode = false; }
    }
    applyModeToggle();
    logEvent(`mode -> ${mode}`);
    render();
  });
});

// feedback via mailto (version + OS + log tail), nothing sent silently
document.getElementById("feedback-btn").addEventListener("click", async () => {
  try {
    const fb = await invoke("collect_feedback");
    const subject = `ProofCat feedback v${fb.version}`;
    const body =
      `${t("feedbackIntro")}\n\n\n\n` +
      `--- system ---\nVersion: ${fb.version}\nOS: ${fb.os}\nLogs folder: ${fb.log_dir}\n\n` +
      `--- recent logs ---\n${(fb.logs || "").slice(-1500)}\n`;
    const url = `mailto:${FEEDBACK_EMAIL}?subject=${encodeURIComponent(subject)}&body=${encodeURIComponent(body)}`;
    await invoke("open_url", { url });
    logEvent("feedback mail opened");
  } catch (e) {
    logEvent(`feedback failed: ${e}`, "error");
  }
});
// open logs folder in file manager
document.getElementById("logs-btn").addEventListener("click", async () => {
  try {
    const fb = await invoke("collect_feedback");
    if (fb.log_dir) await invoke("open_url", { url: `file://${fb.log_dir}` });
    logEvent("open logs folder");
  } catch (e) {
    logEvent(`open logs failed: ${e}`, "error");
  }
});

// ======================= auto-update =======================
let updateAvail = null;
let promptedUpdateVersion = null;
function setUpdateBtn() {
  const label = document.querySelector("#update-btn .btn-label");
  label.textContent = updateAvail ? `${t("updInstall")} v${updateAvail.version}` : t("checkUpdates");
}
async function checkUpdate(manual) {
  const st = document.getElementById("update-status");
  if (manual) st.textContent = t("updChecking");
  try {
    const info = await invoke("check_update");
    if (info && info.available) {
      updateAvail = info;
      st.textContent = `${t("updAvailable")} v${info.version}`;
      document.querySelectorAll(".settings-trigger").forEach((button) => button.classList.add("has-update"));
      if (!manual && promptedUpdateVersion !== info.version) {
        promptedUpdateVersion = info.version;
        window.setTimeout(() => {
          if (updateAvail?.version === info.version && window.confirm(t("updPrompt").replace("{version}", `v${info.version}`))) {
            installUpdate();
          }
        }, 0);
      }
    } else {
      updateAvail = null;
      if (manual) st.textContent = t("updUpToDate");
      document.querySelectorAll(".settings-trigger").forEach((button) => button.classList.remove("has-update"));
    }
  } catch (e) {
    updateAvail = null;
    if (manual) st.textContent = `⚠ ${t("updError")}`;
    logEvent(`update check failed: ${e}`, "warn");
  }
  setUpdateBtn();
}
async function installUpdate() {
  const st = document.getElementById("update-status");
  st.textContent = t("updInstalling");
  logEvent("update install started");
  try {
    await invoke("install_update"); // app restarts on success; no return
  } catch (e) {
    st.textContent = `⚠ ${typeof e === "string" ? e : "install error"}`;
    logEvent(`update install failed: ${e}`, "error");
  }
}
document.getElementById("update-btn").addEventListener("click", () => {
  if (updateAvail) installUpdate(); else checkUpdate(true);
});

// settings modal
const settingsModal = document.getElementById("settings-modal");
function openSettings() {
  settingsModal.hidden = false;
  logEvent("settings opened");
  invoke("collect_feedback").then((fb) => {
    document.getElementById("about-version").textContent = "v" + fb.version;
    document.getElementById("about-path").textContent = fb.log_dir;
  }).catch(() => {});
  invoke("crash_get_opt_in").then((on) => {
    document.getElementById("crash-toggle").checked = !!on;
  }).catch(() => {});
}
const closeSettings = () => { settingsModal.hidden = true; };
document.querySelectorAll(".settings-trigger").forEach((button) => button.addEventListener("click", openSettings));
document.getElementById("settings-close").addEventListener("click", closeSettings);
settingsModal.addEventListener("click", (e) => { if (e.target === settingsModal) closeSettings(); });
window.addEventListener("keydown", (e) => { if (e.key === "Escape" && !settingsModal.hidden) closeSettings(); });

// crash reporting opt-in (default OFF; применяется при следующем запуске)
document.getElementById("crash-toggle").addEventListener("change", (e) => {
  const enabled = e.target.checked;
  invoke("crash_set_opt_in", { enabled })
    .then(() => logEvent("crash opt-in " + (enabled ? "on" : "off")))
    .catch((err) => logEvent("crash opt-in failed: " + err, "error"));
});

// Мост ошибок фронтенда → бэкенд. Локальный лог всегда; в sentry только при opt-in.
window.addEventListener("error", (e) => {
  invoke("crash_report_js", {
    message: String((e && e.message) || "unknown error"),
    source: (e && e.filename) || null,
    line: (e && e.lineno) || null,
    stack: (e && e.error && e.error.stack) ? String(e.error.stack) : null,
  }).catch(() => {});
});
window.addEventListener("unhandledrejection", (e) => {
  const r = e && e.reason;
  invoke("crash_report_js", {
    message: "unhandledrejection: " + String((r && r.message) || r || "unknown"),
    source: null,
    line: null,
    stack: (r && r.stack) ? String(r.stack) : null,
  }).catch(() => {});
});

// custom spec profile editor (Фаза 4)
const specModal = document.getElementById("spec-modal");
const csEls = () => ({
  name: document.getElementById("cs-name"), target: document.getElementById("cs-target"),
  tol: document.getElementById("cs-tol"), tp: document.getElementById("cs-tp"),
  sr: document.getElementById("cs-sr"), fps: document.getElementById("cs-fps"),
  err: document.getElementById("cs-err"), save: document.getElementById("cs-save"), del: document.getElementById("cs-del"),
});
function csValidate() {
  const e = csEls();
  const nameOk = e.name.value.trim().length > 0;
  const tgtOk = num(e.target.value) != null;
  const tol = num(e.tol.value); const tolOk = tol != null && tol >= 0;
  const ok = nameOk && tgtOk && tolOk;
  e.save.disabled = !ok;
  e.err.textContent = ok ? "" : `${t("specMissing")}: ` + [!nameOk && t("specFName"), !tgtOk && t("specFTarget"), !tolOk && t("specFTol")].filter(Boolean).join(", ");
  return ok;
}
function openSpecEditor() {
  const e = csEls();
  let j = null; try { j = JSON.parse(localStorage.getItem("customSpec") || "null"); } catch {}
  e.name.value = j?.name || ""; e.target.value = j?.target ?? ""; e.tol.value = j?.tol ?? "";
  e.tp.value = (j && j.tpMax != null) ? j.tpMax : ""; e.sr.value = j?.sampleRate ? String(j.sampleRate) : "";
  e.fps.checked = !!(j && j.checkFps); e.del.hidden = !j;
  csValidate();
  specModal.hidden = false;
}
const closeSpecEditor = () => { specModal.hidden = true; };
document.getElementById("spec-mclose").addEventListener("click", closeSpecEditor);
specModal.addEventListener("click", (e) => { if (e.target === specModal) closeSpecEditor(); });
window.addEventListener("keydown", (e) => { if (e.key === "Escape" && !specModal.hidden) closeSpecEditor(); });
["cs-name", "cs-target", "cs-tol"].forEach((id) => document.getElementById(id).addEventListener("input", csValidate));
document.getElementById("cs-save").addEventListener("click", () => {
  if (!csValidate()) return;
  const e = csEls();
  const prof = {
    name: e.name.value.trim(), target: num(e.target.value), tol: num(e.tol.value),
    tpMax: e.tp.value.trim() === "" ? null : num(e.tp.value),
    sampleRate: e.sr.value ? +e.sr.value : null, checkFps: e.fps.checked,
  };
  localStorage.setItem("customSpec", JSON.stringify(prof));
  files.forEach((file) => {
    if (file.specRunProfile === "custom") {
      delete file.specRunProfile;
      delete file.specResult;
    }
  });
  specProfile = "custom"; localStorage.setItem("specProfile", specProfile);
  closeSpecEditor(); fillSpecOptions(); render();
  logEvent("custom spec saved");
});
document.getElementById("cs-del").addEventListener("click", () => {
  localStorage.removeItem("customSpec");
  files.forEach((file) => {
    if (file.specRunProfile === "custom") {
      delete file.specRunProfile;
      delete file.specResult;
    }
  });
  if (specProfile === "custom") { specProfile = "none"; localStorage.setItem("specProfile", specProfile); }
  closeSpecEditor(); fillSpecOptions(); render();
  logEvent("custom spec deleted");
});

// drag & drop
const overlay = document.getElementById("drop-overlay");
listen("tauri://drag-enter", () => { overlay.hidden = false; });
listen("tauri://drag-leave", () => { overlay.hidden = true; });
listen("tauri://drag-drop", (e) => {
  overlay.hidden = true;
  const paths = e?.payload?.paths || [];
  if (paths.length) addPaths(paths);
});

applyTheme();
applyLang();
checkUpdate(false); // тихая проверка при запуске; ничего не ставит без клика
