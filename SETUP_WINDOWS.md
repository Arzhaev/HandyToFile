# HandyToFile — установка с нуля на Windows

Пошаговая инструкция для запуска проекта на чистой машине Windows.
Предполагается, что ничего из инструментов разработки ещё не установлено.

---

## Что нужно установить

| Инструмент | Зачем |
|---|---|
| Git | клонирование репозитория |
| Rust + rustup | компиляция бэкенда |
| Visual Studio C++ Build Tools | компилятор C/C++, нужен для сборки Rust-крейтов и whisper.cpp |
| Node.js LTS | сборка фронтенда |
| Vulkan SDK | GPU-ускорение для Whisper |
| WebView2 Runtime | встроенный браузер Tauri (обычно уже есть на Windows 10/11) |

> **LLVM не нужен.** На Windows биндинги для whisper уже сгенерированы заранее — libclang не требуется.

---

## Шаг 1 — Git

Скачать: https://git-scm.com/download/win

Установить с настройками по умолчанию. После установки убедиться:

```powershell
git --version
```

---

## Шаг 2 — Rust

Скачать `rustup-init.exe`: https://rustup.rs/

Запустить, выбрать вариант `1) Proceed with standard installation`.

После установки открыть **новое** окно PowerShell и проверить:

```powershell
rustc --version
cargo --version
```

---

## Шаг 3 — Visual Studio C++ Build Tools

> Если уже установлен полный Visual Studio 2019/2022 с компонентом "Desktop development with C++" — этот шаг пропустить.

Скачать Build Tools: https://visualstudio.microsoft.com/visual-cpp-build-tools/

При установке выбрать компонент **"Desktop development with C++"**. Остальное по умолчанию.

Установка занимает ~5–10 минут и ~5 ГБ места.

---

## Шаг 4 — Node.js LTS

Скачать: https://nodejs.org/ (кнопка "LTS")

Установить с настройками по умолчанию.

Проверить:

```powershell
node --version
npm --version
```

---

## Шаг 5 — Vulkan SDK

Скачать: https://vulkan.lunarg.com/sdk/home#windows

Нажать "Download" для Windows, установить с настройками по умолчанию.

По умолчанию устанавливается в `C:\VulkanSDK\<версия>\`.

Проверить (подставить свою версию):

```powershell
ls 'C:\VulkanSDK\'
# должна быть папка, например C:\VulkanSDK\1.4.341.1\
```

---

## Шаг 6 — Клонировать репозиторий

```powershell
git clone https://github.com/<your-username>/HandyToFile.git C:\projects\HandyToFile
```

---

## Шаг 7 — Установить зависимости фронтенда

```powershell
cd C:\projects\HandyToFile
npm install --ignore-scripts
```

---

## Шаг 8 — Проверить сборку бэкенда

```powershell
cd C:\projects\HandyToFile\src-tauri
$env:VULKAN_SDK = (Get-Item 'C:\VulkanSDK\*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
cargo check
```

Первый раз займёт 5–15 минут (скачивает и компилирует зависимости).
В конце должно быть `Finished`.

---

## Шаг 9 — Запустить приложение

```powershell
powershell -ExecutionPolicy Bypass -File C:\projects\HandyToFile\run-dev.ps1
```

Или вручную:

```powershell
cd C:\projects\HandyToFile
$env:VULKAN_SDK = (Get-Item 'C:\VulkanSDK\*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
npm run tauri dev
```

Первый запуск компилирует весь бэкенд — ожидать 5–15 минут.
Последующие запуски значительно быстрее.

---

## Шаг 10 — Скачать модель распознавания речи

При первом запуске приложение откроется на экране онбординга.
Если автоматическое скачивание не работает (Windows Firewall или прокси) — скачать вручную:

1. Скачать файл через браузер:
   - **Whisper Turbo** (рекомендуется, нужен GPU): `https://blob.handy.computer/ggml-large-v3-turbo.bin`
   - **Whisper Medium** (работает на CPU): `https://blob.handy.computer/whisper-medium-q4_1.bin`

2. Положить `.bin`-файл в папку:
   ```
   C:\Users\<имя_пользователя>\AppData\Roaming\com.pais.handy\models\
   ```
   Если папки `models` нет — создать вручную.

3. Перезапустить приложение. Модель появится как "Downloaded".

---

## Переменная окружения

Переменная `VULKAN_SDK` нужна **только во время сборки**. Для обычной работы приложения она не нужна.

Чтобы не устанавливать её вручную каждый раз — уже есть готовый скрипт `run-dev.ps1` в корне репозитория.

Или добавить постоянно через System Properties → Environment Variables:

| Переменная | Значение |
|---|---|
| `VULKAN_SDK` | `C:\VulkanSDK\1.4.341.1` (подставить свою версию) |

---

## Частые проблемы

### `error: could not find Cargo.toml`
Запускаете команду не из папки проекта. Сначала `cd C:\projects\HandyToFile`.

### Модели не скачиваются из приложения
Windows Firewall может блокировать исходящие соединения для `handy.exe`.
Решение: разрешить в фаерволе или скачать модели вручную (см. Шаг 10).

### Сборка зависает или падает на whisper.cpp
`VULKAN_SDK` не установлена. Убедиться что Vulkan SDK установлен и путь корректный:
```powershell
echo $env:VULKAN_SDK
ls "$env:VULKAN_SDK\Lib\vulkan-1.lib"
```

---

## Итоговый чеклист

- [ ] Git установлен, `git --version` работает
- [ ] Rust установлен, `cargo --version` работает
- [ ] Visual Studio C++ Build Tools установлены
- [ ] Node.js установлен, `npm --version` работает
- [ ] Vulkan SDK установлен
- [ ] `npm install --ignore-scripts` выполнен
- [ ] `cargo check` проходит без ошибок
- [ ] `run-dev.ps1` запускает приложение
- [ ] Модель речи выбрана и доступна
