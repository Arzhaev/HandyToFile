# HandyToFile

HandyToFile — локальный десктопный форк [Handy](https://github.com/cjpais/handy) для распознавания речи в текст.

Форк сохраняет оригинальный рабочий процесс Handy:

- нажать глобальный шорткат
- записать с микрофона
- транскрибировать локально
- вставить результат в активное поле ввода

К этому добавлено одно сфокусированное расширение: тот же конвейер захвата голоса может также дописывать текст в обычные Markdown-файлы через отдельные горячие клавиши.

Это по-прежнему небольшой локальный инструмент захвата, а не приложение для заметок и не хаб облачных интеграций.

См. также:

- [SETUP_WINDOWS.md](SETUP_WINDOWS.md) — установка с нуля на Windows
- [DEPLOYMENT_PLAN.md](DEPLOYMENT_PLAN.md)
- [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md)

## Что добавляет форк

- настраиваемые действия захвата
- несколько профилей распознавания / постобработки
- отдельные горячие клавиши для вставки и захвата в файл
- дозапись в Markdown-файлы для:
  - заметок
  - задач
  - идей
  - покупок
- автоматическое создание отсутствующих директорий и файлов
- всплывающие уведомления об успехе и ошибках при записи в файл

## Основные сценарии использования

### Обычная диктовка

Нажать основной шорткат Handy, произнести текст, отпустить шорткат — транскрибированный текст вставится в текущее активное поле ввода.

### Быстрый захват в Markdown

Нажать специальный шорткат для заметок, задач, идей или покупок, произнести текст, отпустить шорткат — распознанный текст дописывается в настроенный Markdown-файл вместо вставки в активное окно.

## Модель действий захвата

Форк вводит три основные сущности конфигурации:

- `RecognitionProfile`
- `OutputTarget`
- `CaptureAction`

Каждое действие захвата определяет:

- какой шорткат его запускает
- какой профиль распознавания использует
- куда направляет результат

Поддерживаемые выходные цели:

- `paste`
- `append_file`

## Профили распознавания

Конфигурация по умолчанию включает:

- `ru_mixed`
- `ru_only`
- `en_only`
- `raw`

Назначение дефолтных профилей:

- `ru_mixed`: предпочитать русский вывод, сохраняя корректные английские технические термины
- `ru_only`: смещать вывод в сторону русских формулировок
- `en_only`: смещать вывод в сторону английского
- `raw`: минимальная очистка, максимальная близость к сырому результату распознавания

Профили хранятся в конфиге и не привязаны жёстко к одному рабочему процессу.

## Горячие клавиши по умолчанию

### Windows / Linux

- `Ctrl+Space` — вставить в активное окно
- `Ctrl+Shift+Space` — режим вставки с постобработкой (legacy)
- `Ctrl+Alt+N` — дозапись в файл заметок
- `Ctrl+Alt+T` — дозапись в файл задач
- `Ctrl+Alt+I` — дозапись в файл идей
- `Ctrl+Alt+S` — дозапись в файл покупок
- `Escape` — отменить текущую запись

### macOS

- `Option+Space` — вставить в активное окно
- `Option+Shift+Space` — режим вставки с постобработкой (legacy)
- `Command+Option+N` — дозапись в файл заметок
- `Command+Option+T` — дозапись в файл задач
- `Command+Option+I` — дозапись в файл идей
- `Command+Option+S` — дозапись в файл покупок
- `Escape` — отменить текущую запись

## Форматы Markdown-вывода

### Заметки

```md
- YYYY-MM-DD HH:mm — текст
```

### Задачи

```md
- [ ] YYYY-MM-DD HH:mm — текст
```

### Покупки

```md
- [ ] YYYY-MM-DD HH:mm — текст
```

### Идеи

```md
## YYYY-MM-DD HH:mm
текст

```

## Пути Markdown-файлов по умолчанию

Форк предзаполняет редактируемые пути по умолчанию:

- `C:/Users/user/Documents/HandyToFile/notes.md`
- `C:/Users/user/Documents/HandyToFile/tasks.md`
- `C:/Users/user/Documents/HandyToFile/ideas.md`
- `C:/Users/user/Documents/HandyToFile/shopping.md`

Папки создаются автоматически при первой записи. Пути меняются в Settings → Capture File Paths.

## Конфигурация

Приложение хранит настройки в директории данных Handy. Ключевые поля для этого форка:

- `bindings`
- `recognition_profiles`
- `capture_actions`
- `notes_path`
- `tasks_path`
- `ideas_path`
- `shopping_path`

Пример структуры:

```json
{
  "notes_path": "C:/Users/user/Documents/HandyToFile/notes.md",
  "tasks_path": "C:/Users/user/Documents/HandyToFile/tasks.md",
  "ideas_path": "C:/Users/user/Documents/HandyToFile/ideas.md",
  "shopping_path": "C:/Users/user/Documents/HandyToFile/shopping.md",
  "recognition_profiles": [
    {
      "id": "ru_mixed",
      "name": "Russian mixed",
      "language_hint": "ru",
      "instruction_prompt": "Keep the output primarily in Russian while preserving valid English technical terms.",
      "cleanup_options": {
        "trim_whitespace": true,
        "collapse_whitespace": true,
        "preserve_newlines": false,
        "capitalize_first_letter": true
      }
    }
  ],
  "capture_actions": [
    {
      "id": "default_paste_ru",
      "binding_id": "transcribe",
      "profile_id": "ru_mixed",
      "enabled": true,
      "output_target": {
        "type": "paste"
      }
    },
    {
      "id": "quick_note_ru",
      "binding_id": "append_to_notes_file",
      "profile_id": "ru_mixed",
      "enabled": true,
      "output_target": {
        "type": "append_file",
        "file_slot": "notes",
        "template_type": "note_bullet"
      }
    }
  ]
}
```

## Зависимости для разработки

Для разработки и релизных сборок на Windows:

- Rust через `rustup`
- Visual Studio C++ build tools
- Node.js LTS
- WebView2 Runtime
- Vulkan SDK
- Git

> **LLVM не нужен.** На Windows биндинги для whisper уже сгенерированы заранее — libclang не требуется.

Переменная окружения, необходимая во время сборки:

```powershell
$env:VULKAN_SDK = 'C:\VulkanSDK\1.4.341.1'
```

Подробная инструкция с нуля: [SETUP_WINDOWS.md](SETUP_WINDOWS.md)

## Команды разработки

```powershell
# Установка зависимостей
npm install --ignore-scripts

# Запуск в режиме разработки (устанавливает VULKAN_SDK автоматически)
powershell -ExecutionPolicy Bypass -File run-dev.ps1

# Проверка бэкенда
$env:VULKAN_SDK = 'C:\VulkanSDK\1.4.341.1'
cargo check
cargo test

# Сборка релизного пакета
$env:VULKAN_SDK = 'C:\VulkanSDK\1.4.341.1'
npm run tauri build
```

## Текущий статус локальной сборки

Актуальная проверка на Windows:

- `cargo check` проходит
- `cargo test` проходит
- `npm run build` проходит

## Развёртывание у нового пользователя

На обычной пользовательской машине не нужно разворачивать репозиторий и устанавливать тулчейн сборки.

Разворачивайте собранный пакет приложения.

### Что нужно на машине нового пользователя

- упакованное приложение HandyToFile
- WebView2 Runtime на Windows
- разрешение на микрофон
- разрешение на доступность / управление вводом, если требуется для автовставки

### Что обычно не нужно

- Rust
- Cargo
- Node.js
- Visual Studio build tools
- LLVM
- Vulkan SDK
- исходный код репозитория

### Шаги первого запуска для нового пользователя

1. Установить упакованное приложение.
2. Запустить HandyToFile.
3. При запросе предоставить разрешение на микрофон.
4. При необходимости предоставить разрешение на доступность или управление вводом.
5. Скачать или выбрать локальную модель распознавания речи.
6. Проверить дефолтный шорткат вставки.
7. Проверить шорткаты захвата в Markdown.
8. При необходимости скорректировать пути к файлам заметок/задач/идей/покупок в настройках.

## Примечания к релизу форка

Текущий MVP HandyToFile ориентирован на быстрый локальный захват:

- диктовка в активное окно остаётся поведением по умолчанию
- Markdown-цели захвата добавлены без замены оригинального рабочего процесса
- профили и действия управляются через конфиг для возможности дальнейшего расширения

Форк не стремится стать:

- полноценным приложением для заметок
- платформой второго мозга
- сервисом облачной синхронизации
- интеграционным слоем с Telegram или агентами
