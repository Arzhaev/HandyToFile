# HandyToFile

> Форк проекта [Handy](https://github.com/cjpais/Handy) — бесплатного офлайн-приложения для речевого ввода текста.
> Добавлена запись расшифровок в Markdown-файлы, профили распознавания и горячие клавиши для захвата по категориям.

**Платформы:** Windows · macOS · Linux

---

## Что добавлено в этом форке

- **Запись в файлы** — расшифровки сохраняются в Markdown-файлы (заметки, задачи, идеи, покупки)
- **Профили распознавания** — `ru_mixed`, `ru_only`, `en_only`, `raw`
- **Горячие клавиши для захвата** — `Ctrl+Alt+N/T/I/S` для записи по категориям
- **Windows-совместимость** — скрипт запуска, инструкции для чистой установки

Подробнее: [README_HANDYTOFILE.md](README_HANDYTOFILE.md)

---

## Быстрый старт (Windows)

Полная инструкция с нуля: [SETUP_WINDOWS.md](SETUP_WINDOWS.md)

**Что нужно установить:** Git, Rust, Visual Studio C++ Build Tools, Node.js, LLVM, Vulkan SDK

```powershell
git clone https://github.com/<your-username>/HandyToFile.git C:\projects\HandyToFile
cd C:\projects\HandyToFile
npm install --ignore-scripts
powershell -ExecutionPolicy Bypass -File run-dev.ps1
```

При первом запуске скачать модель распознавания речи в настройках (раздел Models).
Если автозагрузка не работает — [инструкция по ручной установке моделей](SETUP_WINDOWS.md#шаг-11--скачать-модель-распознавания-речи).

---

## Горячие клавиши (Windows / Linux)

| Действие | Клавиши |
|---|---|
| Записать → вставить в курсор | `Ctrl+Space` |
| Записать → постобработка | `Ctrl+Shift+Space` |
| Записать → файл заметок | `Ctrl+Alt+N` |
| Записать → файл задач | `Ctrl+Alt+T` |
| Записать → файл идей | `Ctrl+Alt+I` |
| Записать → файл покупок | `Ctrl+Alt+S` |
| Отмена | `Escape` |

Режим по умолчанию — **Push-to-Talk** (держать клавишу). Toggle можно включить в настройках.

---

## Настройки

Файл настроек: `%APPDATA%\com.pais.handy\settings.json`

Пути к Markdown-файлам настраиваются в Settings → Capture File Paths.

По умолчанию:
```
D:/Obsidian/00 Inbox/Voice Notes.md   (заметки)
D:/Obsidian/00 Inbox/Tasks.md         (задачи)
D:/Obsidian/00 Inbox/Ideas.md         (идеи)
D:/Obsidian/00 Inbox/Shopping.md      (покупки)
```

---

## Разработка

```powershell
# Запуск в режиме разработки (Windows)
powershell -ExecutionPolicy Bypass -File run-dev.ps1

# Или вручную
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
$env:VULKAN_SDK    = 'C:\VulkanSDK\1.4.341.1'
npm run tauri dev
```

Подробности: [DEPLOYMENT_PLAN.md](DEPLOYMENT_PLAN.md) · [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)

---

## Лицензии

- Этот форк: [MIT License](LICENSE)
- Оригинальный Handy: [MIT License](https://github.com/cjpais/Handy/blob/main/LICENSE), © 2025 CJ Pais
- whisper.cpp: [MIT License](vendor/whisper-rs-sys/whisper.cpp/LICENSE), © The ggml authors
- whisper-rs-sys: [Unlicense](vendor/whisper-rs-sys/LICENSE) (public domain)

## Благодарности

- **CJ Pais** — оригинальный проект [Handy](https://github.com/cjpais/Handy)
- **OpenAI** — модель Whisper
- **ggml / whisper.cpp** — инференс на CPU/GPU
- **Silero** — VAD (определение голосовой активности)
- **Tauri** — фреймворк приложения
