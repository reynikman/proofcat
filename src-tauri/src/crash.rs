//! Crash reporting — opt-in клиент к своему GlitchTip (Sentry-совместимый), НЕ чужое
//! облако. Полный контракт: docs/crash-reporting.md.
//!
//! Три железных правила:
//!   1. Opt-in, default OFF. Пока пользователь не включил отправку в Настройках —
//!      sentry не инициализируется, наружу не уходит ничего.
//!   2. DSN не хардкодится: `option_env!("GLITCHTIP_DSN")` задаётся защищённой
//!      средой сборки. Пусто (форк / сборка без секрета) → не инициализируемся.
//!   3. Scrub PII в `before_send`: пути и имена файлов клиента (NDA съёмок) вычищаются.
//!      Наружу — только версия, ОС, тип ошибки и структура стека.
//!
//! Честная граница: базовый sentry-rust НЕ кэширует события на диск. Если площадка
//! офлайн, паник-событие живёт в памяти и флашится на выходе; при отсутствии сети
//! может не доехать — но локальный лог (`tauri-plugin-log`) пишет его ВСЕГДА и не
//! зависит от этого модуля. Настоящий store-and-forward для жёстких крашей даёт
//! minidump (см. следующий инкремент A2).

use std::path::Path;
use std::sync::Arc;

/// Файл-флаг opt-in в app_config_dir. Пишется фронтом через Tauri-команду.
/// Содержимое `1`/`true` = включено; всё прочее (в т.ч. отсутствие) = выключено.
pub const OPT_IN_FILE: &str = "crash_reporting_enabled";

/// DSN from protected build env. None in ordinary source builds and forks.
const DSN: Option<&str> = option_env!("GLITCHTIP_DSN");

/// Прочитать opt-in. Default OFF: нет файла / ошибка чтения / не `1|true` → false.
pub fn opt_in_enabled(config_dir: &Path) -> bool {
    match std::fs::read_to_string(config_dir.join(OPT_IN_FILE)) {
        Ok(s) => matches!(s.trim(), "1" | "true"),
        Err(_) => false,
    }
}

/// Записать opt-in-флаг (из Tauri-команды при переключении тумблера в Настройках).
/// Пишем явный `0` вместо удаления — однозначное «выключено».
pub fn set_opt_in(config_dir: &Path, enabled: bool) -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir)?;
    std::fs::write(
        config_dir.join(OPT_IN_FILE),
        if enabled { "1" } else { "0" },
    )
}

/// Инициализировать sentry, если (а) DSN вшит и непуст и (б) opt-in включён.
/// Иначе None (полностью инертно). Guard держать живым весь рантайм — Drop флашит очередь.
pub fn init(config_dir: &Path) -> Option<sentry::ClientInitGuard> {
    let dsn = DSN?;
    if dsn.is_empty() || !opt_in_enabled(config_dir) {
        return None;
    }
    let options = sentry::ClientOptions {
        dsn: dsn.parse().ok(),
        release: sentry::release_name!(),
        // NDA: не слать имя хоста/пользователя автоматически.
        send_default_pii: false,
        server_name: None,
        before_send: Some(Arc::new(|event| Some(scrub_event(event)))),
        ..Default::default()
    };
    let guard = sentry::init(options);
    if guard.is_enabled() {
        log::info!(
            "crash reporting ON (opt-in) release={:?}",
            sentry::release_name!()
        );
        Some(guard)
    } else {
        // DSN не распарсился — не держим бесполезный guard.
        None
    }
}

/// Вычистить пути/имена файлов клиента из события ПЕРЕД отправкой.
/// Тип ошибки и структуру стека оставляем (нужны для диагностики), текстовые поля чистим.
fn scrub_event(mut event: sentry::protocol::Event<'static>) -> sentry::protocol::Event<'static> {
    if let Some(msg) = event.message.take() {
        event.message = Some(scrub_text(&msg));
    }
    if let Some(mut le) = event.logentry.take() {
        le.message = scrub_text(&le.message);
        event.logentry = Some(le);
    }
    for exc in event.exception.values.iter_mut() {
        if let Some(v) = exc.value.take() {
            exc.value = Some(scrub_text(&v));
        }
    }
    for bc in event.breadcrumbs.values.iter_mut() {
        if let Some(m) = bc.message.take() {
            bc.message = Some(scrub_text(&m));
        }
    }
    // extra может содержать пути в строковых значениях — чистим их.
    for value in event.extra.values_mut() {
        if let Some(s) = value.as_str() {
            *value = serde_json::Value::String(scrub_text(s));
        }
    }
    event
}

/// Заменить путеподобные токены на `<path>` / `<path:.ext>`, сохранив расширение
/// для диагностики. Токен, не похожий на путь, остаётся как есть.
fn scrub_text(s: &str) -> String {
    s.split_whitespace()
        .map(|tok| {
            if looks_like_path(tok) {
                redact_path(tok)
            } else {
                tok.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn looks_like_path(tok: &str) -> bool {
    tok.contains('/') || tok.contains('\\')
}

fn redact_path(tok: &str) -> String {
    // снять обрамляющую пунктуацию/кавычки
    let trimmed = tok
        .trim_matches(|c: char| matches!(c, '"' | '\'' | '(' | ')' | '[' | ']' | ',' | ';' | ':'));
    match std::path::Path::new(trimmed)
        .extension()
        .and_then(|e| e.to_str())
    {
        Some(e)
            if !e.is_empty() && e.len() <= 5 && e.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            format!("<path:.{e}>")
        }
        _ => "<path>".to_string(),
    }
}

/// Переслать ошибку фронтенда (window.onerror / unhandledrejection) в sentry.
/// No-op, если клиент не активен. Пути/имена в message/stack отскрабит `before_send`.
pub fn capture_js_error(
    message: &str,
    source: Option<&str>,
    line: Option<u32>,
    stack: Option<&str>,
) {
    let mut event = sentry::protocol::Event {
        level: sentry::protocol::Level::Error,
        logger: Some("frontend".to_string()),
        message: Some(message.to_string()),
        ..Default::default()
    };
    if let Some(s) = source {
        event.extra.insert("source".to_string(), s.into());
    }
    if let Some(l) = line {
        event.extra.insert("line".to_string(), l.into());
    }
    if let Some(st) = stack {
        event.extra.insert("stack".to_string(), st.into());
    }
    sentry::capture_event(event);
}

/// Добавить breadcrumb (хлебную крошку) — контекст к будущему событию. No-op без клиента.
/// Крошки цепляются к событию при отправке и проходят тот же `before_send`-scrub.
pub fn breadcrumb(category: &str, message: String) {
    sentry::add_breadcrumb(sentry::protocol::Breadcrumb {
        category: Some(category.to_string()),
        message: Some(message),
        level: sentry::protocol::Level::Info,
        ..Default::default()
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_strips_unix_path_keeps_extension() {
        let msg = "Cannot open source file: /Users/operator/ShootX/clientA/clip001.mov (os error 2)";
        let out = scrub_text(msg);
        assert!(!out.contains("operator"), "username leaked: {out}");
        assert!(!out.contains("clientA"), "client name leaked: {out}");
        assert!(!out.contains("clip001"), "filename leaked: {out}");
        assert!(
            out.contains("<path:.mov>"),
            "extension not preserved: {out}"
        );
        assert!(
            out.contains("Cannot open source file"),
            "error text lost: {out}"
        );
    }

    #[test]
    fn scrub_strips_windows_path() {
        let out = scrub_text("Failed to rename D:\\Shoot\\take\\a.r3d -> E:\\backup\\a.r3d");
        assert!(!out.contains("Shoot"), "windows path leaked: {out}");
        assert!(!out.contains("backup"), "windows path leaked: {out}");
        assert!(out.contains("<path:.r3d>"), "ext not kept: {out}");
    }

    #[test]
    fn scrub_leaves_plain_message_untouched() {
        let msg = "Size mismatch after copy: source=1024 copied=512";
        assert_eq!(scrub_text(msg), msg);
    }

    #[test]
    fn scrub_redacts_path_without_extension() {
        let out = scrub_text("dir /Users/example/secret missing");
        assert!(out.contains("<path>"), "bare path not redacted: {out}");
        assert!(!out.contains("example"));
        assert!(out.contains("dir") && out.contains("missing"));
    }

    #[test]
    fn opt_in_defaults_off_and_roundtrips() {
        let dir = tempfile::tempdir().unwrap();
        assert!(!opt_in_enabled(dir.path()), "default must be OFF");
        set_opt_in(dir.path(), true).unwrap();
        assert!(opt_in_enabled(dir.path()));
        set_opt_in(dir.path(), false).unwrap();
        assert!(!opt_in_enabled(dir.path()), "explicit 0 must read as OFF");
    }
}
