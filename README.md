# telegramka-rs API

Бэкенд сервиса telegramka.

## Настройка окружения

### Запуск базы данных и Redis

Для запуска необходимых сервисов (PostgreSQL и Redis) можно использовать Docker или Podman.

**С помощью Docker:**
```bash
# Запуск PostgreSQL
docker run -d \
  --name telegramka \
  -e POSTGRES_DB=telegramka \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  postgres:16

# Запуск Redis
docker run -d \
  --name redis \
  -p 6379:6379 \
  redis:latest
```

**С помощью Podman:**
```bash
# Запуск PostgreSQL
podman run -d \
  --name hits \
  -e POSTGRES_DB=telegramka \
  -e POSTGRES_USER=lexunok \
  -e POSTGRES_PASSWORD=password \
  -p 5432:5432 \
  docker.io/postgres:16

# Запуск Redis
podman run -d \
  --name redis \
  -p 6379:6379 \
  docker.io/redis:latest
```

## Разработка

### Миграции базы данных

Для работы с миграциями используется `sea-orm-cli`.

**Создание новой миграции:**
```bash
sea-orm-cli migrate generate <название_миграции>
```
*Пример:*
```bash
sea-orm-cli migrate generate create_user_table
```

**Применение всех миграций (накатить):**
Для применения всех ожидающих миграций, выполните:
```bash
cargo run -p migration -- up
```

**Полный сброс и применение всех миграций:**
Эта команда удалит все данные в базе данных, а затем применит все миграции с самого начала.
```bash
cargo run -p migration -- fresh
```

### Генерация сущностей (Entities)
```bash
sea-orm-cli generate entity --output-dir ./entity/generated --lib --entity-format dense --with-serde both
```

## Push-уведомления (FCM)

### Переменные окружения

- `FCM_SERVER_KEY` — ключ Firebase Cloud Messaging для отправки push (optional, если отсутствует — push не отправляются).
- `FCM_ENDPOINT` — endpoint FCM (по умолчанию `https://fcm.googleapis.com/fcm/send`).

### Пример payload

```json
{
  "data": {
    "chat_id": "...",
    "user_id": "...",
    "sender_name": "...",
    "sender_nickname": "...",
    "avatar_url": "...",
    "text": "..."
  }
}
```
