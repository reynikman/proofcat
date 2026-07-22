<p align="center">
  <a href="README.md">English</a> · <a href="README.zh-CN.md">中文</a> · <a href="README.ru.md">Русский</a> · <a href="README.ja.md">日本語</a>
</p>

# ProofCat

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/hero-dark.png">
    <img alt="ProofCat — сначала докажи копию, потом используй карту снова" src="docs/assets/hero-light.png">
  </picture>
</p>

<p align="center"><strong>Сначала сделайте две проверенные копии. Потом используйте карту снова.</strong></p>

<p align="center">
  Бесплатный офлайн-инструмент для копирования съёмочных карт на macOS и Windows.<br>
  <a href="https://github.com/reynikman/proofcat/releases/tag/v0.3.0"><strong>Скачать ProofCat 0.3.0</strong></a>
  · <a href="docs/TECHNICAL.md">Техническая документация (английский)</a>
</p>

Съёмка закончена. ProofCat копирует карту на выбранные диски, независимо проверяет
эти копии и даёт ясный ответ. Он никогда ничего не форматирует за вас. Он лишь
показывает, достаточно ли доказательств, чтобы снова использовать карту.

## Один понятный ответ

1. Выберите карту и два диска-приёмника.
2. ProofCat копирует и проверяет каждый нужный файл.
3. Используйте карту повторно только при вердикте **SAFE TO FORMAT**.

При пропавшем файле, неудачной проверке, остановленном задании, заполненном диске
или неоднозначной конфигурации этого вердикта не будет. Две папки на одном
физическом диске не считаются двумя резервными копиями.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/verdict-dark.png">
    <img alt="ProofCat показывает SAFE TO FORMAT только после прохождения всех обязательных проверок" src="docs/assets/verdict-light.png">
  </picture>
</p>

## Для момента после съёмки

- **Офлайн по умолчанию.** Медиа остаётся на вашем компьютере.
- **Два настоящих носителя.** Проверяются устройства, а не только имена папок.
- **Продолжение вместо догадок.** Подключите диск снова и продолжите остановленную работу.
- **Доказательство для передачи.** Сохраняется понятный отчёт о копиях.
- **Не только оффлоад.** В том же приложении есть инспекция медиа и экспорт отчётов.

## Скачать ProofCat

**ProofCat 0.3.0** работает на **macOS Apple Silicon** и **Windows x64**. Скачайте
нужный установщик на [странице релиза](https://github.com/reynikman/proofcat/releases/tag/v0.3.0).

При первом запуске macOS может показать Gatekeeper, а Windows — SmartScreen:
текущий релиз ещё не notarized у Apple и не имеет Windows Authenticode-подписи.
На странице релиза лежат checksums и подписи; их проверка описана в
[технической документации](docs/TECHNICAL.md#installation-and-release-integrity).

## Нужны технические детали?

Простое обещание продукта и инженерные доказательства специально разделены.
Технические документы ниже — на английском:

| Вопрос | Читать |
|---|---|
| Что именно означает `SAFE TO FORMAT`? | [Safety contract](docs/offload-guarantees.md) |
| Как устроены копирование и проверка? | [Technical documentation](docs/TECHNICAL.md) |
| Что показали тесты на настоящем железе? | [Hardware test report](docs/TEST_REPORT.md) |
| Какие есть границы и риски? | [Threat model](docs/threat-model.md) |
| Сравнение с известными инструментами | [Honest comparison](docs/COMPARISON.md) |
| Как собрать проект или внести вклад? | [Contributing](CONTRIBUTING.md) |

## Открытый код и ответственность за вердикт

ProofCat распространяется по [лицензии MIT](LICENSE). Инструмент, который говорит,
можно ли снова использовать карту со съёмкой, должен быть доступен для проверки.
Нашли проблему — откройте [GitHub issue](https://github.com/reynikman/proofcat/issues).
Для проблем безопасности используйте [приватный отчёт](SECURITY.md).
