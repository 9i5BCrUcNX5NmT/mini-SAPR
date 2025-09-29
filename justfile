# Justfile для Bevy проекта

# Запуск с динамической линковкой (быстро)
dev:
    cargo run --features dev

# Сборка проекта
build:
    cargo build

# Сборка релиза
build-release:
    cargo build --release

# Запуск тестов
test:
    cargo test

# Автоматическая пересборка при изменениях
watch:
    cargo watch -x "run"

# Форматирование кода
fmt:
    cargo fmt

# Линтинг
lint:
    cargo clippy

# Проверка безопасности
audit:
    cargo audit

# Очистка
clean:
    cargo clean
