// Offload mode — слив карт с верификацией (tee-copy + хеш + ASC MHL).
//
// Движок портирован из DIT-Pro (https://github.com/WillZ5/DIT-Pro), MIT License,
// Copyright (c) 2026 WillZ. См. NOTICE в корне репозитория. Модули адаптированы под
// Meta Report: берём только offload-ядро (без db/tray/notify/report/cloud DIT-Pro).
//
// Порядок вшивания (инкрементами, каждый проверяется CI):
//   1. hash_engine        — мультиалго один проход (XXH64/XXH3/XXH128/SHA-256/MD5)  [есть]
//   2. mhl                — ASC MHL v2.0 манифест + verify                          [есть]
//   3. copy_engine        — tee-copy 1 read → N write + atomic writer               [есть]
//      volume             — free-space + physical identity + DeviceType             [есть]
//   4. checkpoint         — SQLite WAL, crash-safe resume                           [есть]
//      io_scheduler       — конкуренция IO по типу устройства                       [есть]
//   5. orchestrator + Tauri commands + фронт-вкладка Offload (+ автодетект карт)

pub mod checkpoint;
pub mod copy_engine;
pub mod hash_engine;
pub mod io_scheduler;
pub mod mhl;
pub mod orchestrator;
pub mod report;
pub mod volume;

// Fault-injection / chaos-стенд (симуляция сбоев передачи: read-only dest, диск полон,
// юникод, симлинки, порча байта, resume после краха). Только под тестами.
#[cfg(test)]
mod chaos;
