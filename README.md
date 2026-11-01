# miani

## !!!ВНИМАНИЕ!!! 
проект крайне сырой, написан по большей части с помощью нейросетей, много багов
## СКАЧИВАЙТЕ НА СВОЙ СТРАХ И РИСК 

Минималистичный CLI агент для работы с локальными LLM через LM Studio или Ollama с поддержкой множества режимов работы в стиле Claude Code.

## Возможности

- 🚀 Лёгкий и быстрый CLI интерфейс
- 💬 Потоковый вывод ответов в реальном времени
- 🎨 Цветной вывод для лучшего UX
- 📝 Сохранение истории разговора в рамках сессии
- ⚡ Минимальные зависимости
- 🔧 Bash команды через `!` префикс
- 🛠️ Tool режимы через `@` префикс (как в Claude Code)
- 🤖 Специализированные агенты для поиска и исправления багов
- ⌨️ **Control+C** для остановки ответа AI, двойной **Control+C** для выхода

## Требования

- Rust (1.70+)
- Один из следующих провайдеров LLM:
  - [LM Studio](https://lmstudio.ai/) запущенный локально на порту 1234
  - [Ollama](https://ollama.ai/) запущенный локально на порту 11434

## Установка

```bash
cargo build --release
```

## Поддерживаемые провайдеры

### LM Studio (по умолчанию)
- **Endpoint:** http://localhost:1234/v1
- **Установка:** Скачать с [lmstudio.ai](https://lmstudio.ai/)
- **Запуск:** Открыть LM Studio, загрузить модель и запустить сервер

### Ollama
- **Endpoint:** http://localhost:11434/v1
- **Установка:** Скачать с [ollama.ai](https://ollama.ai/)
- **Запуск:** `ollama serve` в терминале

## Использование

1. Запустите один из провайдеров (LM Studio или Ollama)
2. Запустите CLI агент:

```bash
# LM Studio (по умолчанию)
cargo run

# Ollama
cargo run -- -p ollama

# С флагами
cargo run -- [OPTIONS]
```

### Флаги командной строки

```bash
# Выбор провайдера (lmstudio или ollama)
miani -p ollama

# Выбор конкретной модели
miani -m qwen2.5-coder-7b
miani -p ollama -m llama2

# Продолжить с сохраненного разговора
miani --continue

# Указать кастомный endpoint
miani -e http://localhost:8080/v1
miani -e http://192.168.1.100:11434/v1

# Комбинация флагов
miani -p ollama -m codellama:7b --continue

# Справка
miani --help
```

**Доступные флаги:**
- `-p, --provider <PROVIDER>` - Провайдер (lmstudio или ollama, по умолчанию: lmstudio)
- `-m, --model <MODEL>` - Название модели для использования
- `--continue` - Продолжить с последнего сохраненного разговора
- `-e, --endpoint <ENDPOINT>` - API endpoint (переопределяет дефолтный для провайдера)
- `-h, --help` - Показать справку
- `-V, --version` - Показать версию

### Примеры использования

**LM Studio:**
```bash
# Простой запуск
miani

# С конкретной моделью
miani -m qwen2.5-coder-7b
```

**Ollama:**
```bash
# Базовый запуск
miani -p ollama

# С моделью Llama 2
miani -p ollama -m llama2

# CodeLlama для программирования
miani -p ollama -m codellama:7b

# Mistral
miani -p ollama -m mistral
```

**Кастомные endpoint'ы:**
```bash
# Удаленный LM Studio сервер
miani -e http://192.168.1.100:1234/v1

# Удаленный Ollama сервер
miani -p ollama -e http://192.168.1.100:11434/v1
```

## Режимы работы

### 💬 Chat Mode (по умолчанию)
Просто вводите сообщения для общения с AI:
```
> Привет! Как дела?
AI: Привет! Всё отлично, чем могу помочь?
```

### ⌨️ Управление через Control+C
- **Одинарный Ctrl+C** во время ответа AI - остановить текущий ответ
  - Ответ прерывается и не сохраняется в историю
  - Можно сразу написать новый промпт
  - Работает бесшумно, без дополнительных сообщений
- **Двойной Ctrl+C** (в течение 2 секунд) - выход из программы
  - Быстрый способ завершить работу
  - Работает в любой момент

```
> Расскажи очень длинную историю...
AI: Жил-был однажды...
^C
[Response interrupted]

> какая погода?
AI: Извини, но я не имею доступа к актуальным данным о погоде...
```

### 🔧 Bash Mode (префикс `!`)
Выполняйте bash команды напрямую:
```
> !ls -la
> !git status
> !cargo test
> !cat src/main.rs
```

### 🛠️ Tool Modes (префикс `@`)

#### @file - Чтение файлов
```
> @file src/main.rs
> @file Cargo.toml
```

#### @git - Git операции
```
> @git status       # Статус репозитория
> @git diff          # Изменения
> @git log 10        # Последние 10 коммитов
```

#### @grep - Поиск в файлах
```
> @grep TODO .              # Найти все TODO
> @grep "fn main" src/      # Поиск функции main
```

#### @tree - Структура директории
```
> @tree                # Текущая директория
> @tree src/          # Конкретная директория
```

#### @code - Инструменты для кода
```
> @code format        # Форматирование Rust кода (cargo fmt)
> @code lint          # Проверка линтером (cargo clippy)
> @code review src/   # Обзор кода (будущая функция)
```

## Команды

### Специальные команды:
- `/help` - показать все доступные команды
- `/model` - показать текущую загруженную модель
- `/clear` - очистить историю разговора
- `/exit` или `/quit` - выйти из программы

<<<<<<< HEAD
=======
## Агенты

Проект включает специализированных агентов для работы с кодом:

### 🔍 bug-finder
Находит баги и проблемы в коде:
- Security issues (SQL injection, command injection)
- Потенциальные паники (unwrap, index out of bounds)
- Проблемы с памятью и производительностью
- Логические ошибки

**Использование через Claude Code:**
```
Запусти bug-finder агента для анализа src/
```

### 🔧 bug-fixer
Исправляет найденные баги:
- Proper error handling вместо unwrap/expect
- Добавляет валидацию и проверки
- Исправляет security issues
- Проверяет компиляцию после исправлений

**Использование через Claude Code:**
```
Запусти bug-fixer для исправления багов в src/main.rs
```

>>>>>>> b8152f9 (initial_comit)
## Настройка

По умолчанию агент подключается к:
- Эндпоинт: `http://localhost:1234/v1`
- Модель: `local-model`

Для изменения настроек отредактируйте `src/config.rs`.

## Пример сессии

```
Checking connection to LM Studio...
✓ Connected to LM Studio
Loaded model: qwen2.5-coder-7b-instruct

CLI Agent ready.
  Type your message to chat
  Type ! for bash commands (e.g., !ls -la)
  Type @ for tool modes (e.g., @file, @git, @grep)
  Type /help for all commands

> @git status
On branch main
nothing to commit, working tree clean

> !cargo test
   Compiling niriyabai v0.1.0
    Finished test [unoptimized + debuginfo] target(s) in 1.23s
     Running unittests src/main.rs (target/debug/deps/niriyabai-...)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured

> @file src/config.rs
File: src/config.rs
────────────────────────────────────────────────────────────
pub struct Config {
    pub endpoint: String,
    pub model: String,
}
...
────────────────────────────────────────────────────────────

> Объясни структуру проекта
AI: Этот проект - минималистичный CLI агент для работы с LLM...

> /exit
Goodbye! 👋
```

## Структура проекта

```
niriyabai/
├── src/
│   ├── main.rs       # Точка входа и CLI цикл
│   ├── lmstudio.rs   # Клиент для LM Studio API
│   ├── config.rs     # Конфигурация
│   └── modes.rs      # Tool режимы (@file, @git, и т.д.)
├── .claude/
│   └── agents/       # Специализированные агенты
│       ├── bug-finder.md
│       └── bug-fixer.md
├── Cargo.toml
└── README.md
```

## Технические детали

- **Язык**: Rust 2021 edition
- **Async runtime**: Tokio
- **HTTP client**: Reqwest с поддержкой streaming
- **Минимальные зависимости**: 7 основных крейтов
- **API**: OpenAI-compatible (LM Studio)

## Лицензия

MIT

# pidarasina
# pidarasina
# pidarasina
# pidarasina
