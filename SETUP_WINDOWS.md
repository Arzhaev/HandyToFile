# HandyToFile — установка с нуля на Windows

Это пошаговая инструкция для запуска проекта на чистой машине Windows.
Предполагается, что ничего из инструментов разработки ещё не установлено.

---

## Что нужно установить

| Инструмент | Зачем |
|---|---|
| Git | клонирование репозитория |
| Rust + rustup | компиляция бэкенда |
| Visual Studio C++ Build Tools | компилятор C/C++, нужен для сборки Rust-крейтов |
| Node.js LTS | сборка фронтенда |
| LLVM | библиотека `libclang.dll`, нужна для bindgen |
| Vulkan SDK | GPU-ускорение для Whisper |
| WebView2 Runtime | встроенный браузер Tauri (обычно уже есть на Windows 10/11) |

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

## Шаг 5 — LLVM

Скачать последний релиз `LLVM-*-win64.exe`: https://github.com/llvm/llvm-project/releases

Искать файл вида `LLVM-19.x.x-win64.exe` (или новее).

При установке **обязательно** отметить: **"Add LLVM to the system PATH"** → "For all users".

Проверить:

```powershell
Test-Path 'C:\Program Files\LLVM\bin\libclang.dll'
# должно вернуть True
```

---

## Шаг 6 — Vulkan SDK

Скачать: https://vulkan.lunarg.com/sdk/home#windows

Нажать "Download" для Windows, установить с настройками по умолчанию.

По умолчанию устанавливается в `C:\VulkanSDK\<версия>\`.

Проверить (подставить свою версию):

```powershell
ls 'C:\VulkanSDK\'
# должна быть папка, например C:\VulkanSDK\1.4.341.1\
```

---

## Шаг 7 — Клонировать репозиторий

```powershell
git clone https://github.com/<your-fork>/HandyToFile.git C:\projects\HandyToFile
```

---

## Шаг 8 — Установить зависимости фронтенда

```powershell
cd C:\projects\HandyToFile
npm install --ignore-scripts
```

---

## Шаг 9 — Проверить сборку бэкенда

```powershell
cd C:\projects\HandyToFile\src-tauri
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
$env:VULKAN_SDK    = (Get-Item 'C:\VulkanSDK\*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
cargo check
```

Первый раз займёт 5–15 минут (скачивает и компилирует зависимости).
В конце должно быть `Finished`.

---

## Шаг 10 — Запустить приложение

```powershell
powershell -ExecutionPolicy Bypass -File C:\projects\HandyToFile\run-dev.ps1
```

Или вручную:

```powershell
cd C:\projects\HandyToFile
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
$env:VULKAN_SDK    = (Get-Item 'C:\VulkanSDK\*' | Sort-Object Name -Descending | Select-Object -First 1).FullName
npm run tauri dev
```

Первый запуск компилирует весь бэкенд — ожидать 5–15 минут.
Последующие запуски значительно быстрее.

---

## Шаг 11 — Скачать модель распознавания речи

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

## Переменные окружения

Переменные `LIBCLANG_PATH` и `VULKAN_SDK` нужны **только во время сборки**. Для обычной работы приложения они не нужны.

Чтобы не устанавливать их вручную каждый раз — уже есть готовый скрипт `run-dev.ps1` в корне репозитория.

Или добавить их постоянно через System Properties → Environment Variables:

| Переменная | Значение |
|---|---|
| `LIBCLANG_PATH` | `C:\Program Files\LLVM\bin` |
| `VULKAN_SDK` | `C:\VulkanSDK\1.4.341.1` (подставить свою версию) |

---

## Частые проблемы

### `error: could not find Cargo.toml`
Запускаете команду не из папки проекта. Сначала `cd C:\projects\HandyToFile`.

### `"bun" не является внутренней командой`
В `src-tauri/tauri.conf.json` прописан `bun`, но используется npm.
В этом форке уже исправлено — проверьте что в `tauri.conf.json` стоит `npm run dev`.

### Модели не скачиваются из приложения
Windows Firewall может блокировать исходящие соединения для `handy.exe`.
Решение: разрешить в фаерволе или скачать модели вручную (см. Шаг 11).

### `libclang.dll not found`
Переменная `LIBCLANG_PATH` не установлена или путь неверный.
Проверить: `Test-Path 'C:\Program Files\LLVM\bin\libclang.dll'`.

### Сборка зависает на Vulkan-компоненте
`VULKAN_SDK` не установлена. Убедиться что Vulkan SDK установлен и путь корректный.

---

## Итоговый чеклист

- [ ] Git установлен, `git --version` работает
- [ ] Rust установлен, `cargo --version` работает
- [ ] Visual Studio C++ Build Tools установлены
- [ ] Node.js установлен, `npm --version` работает
- [ ] LLVM установлен, `libclang.dll` доступна
- [ ] Vulkan SDK установлен
- [ ] `npm install --ignore-scripts` выполнен
- [ ] `cargo check` проходит без ошибок
- [ ] `run-dev.ps1` запускает приложение
- [ ] Модель речи выбрана и доступна
