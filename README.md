# Kemenkeu Backend

REST API backend for Kemenkeu family finance tracker.

## Stack

- **Runtime**: Rust + Tokio
- **Framework**: Axum 0.8
- **Database**: MySQL via SQLx
- **Auth**: None (single-user family app)

## Prerequisites

- Rust 1.87+
- MySQL (via Laragon or Docker)
- Database `kemenkeu` created

## Environment Variables

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | `mysql://root:@127.0.0.1:3306/kemenkeu` | MySQL connection string |
| `RUST_LOG` | `info` | Log level |

## Run

```bash
# Local
cargo run

# Docker
docker build -t kemenkeu-backend .
docker run -p 3001:3001 -e DATABASE_URL="mysql://root:@host.docker.internal:3306/kemenkeu" kemenkeu-backend
```

## API Endpoints

### Users
| Method | Endpoint | Description |
|---|---|---|
| GET | `/api/users` | List all users |
| POST | `/api/users` | Create user `{ name }` |
| PUT | `/api/users/:id` | Update user `{ name }` |
| DELETE | `/api/users/:id` | Delete user |

### Categories
| Method | Endpoint | Description |
|---|---|---|
| GET | `/api/categories` | List all (optional `?type=income\|expense`) |
| POST | `/api/categories` | Create `{ name, type, icon }` |
| PUT | `/api/categories/:id` | Update `{ name?, type?, icon? }` |
| DELETE | `/api/categories/:id` | Delete category |

### Transactions
| Method | Endpoint | Description |
|---|---|---|
| GET | `/api/transactions` | List (optional `?person=&category_id=&month=&limit=&offset=`) |
| POST | `/api/transactions` | Create `{ person, amount_cents, category_id, note, date }` |
| PUT | `/api/transactions/:id` | Update fields |
| DELETE | `/api/transactions/:id` | Delete transaction |

### Budgets
| Method | Endpoint | Description |
|---|---|---|
| GET | `/api/budgets` | List (optional `?month=YYYY-MM`) |
| POST | `/api/budgets` | Upsert `{ category_id, person, month, amount_cents }` |
| GET | `/api/budgets/status` | Budget vs actual `?month=YYYY-MM` |
| DELETE | `/api/budgets/:id` | Delete budget |

### Summary
| Method | Endpoint | Description |
|---|---|---|
| GET | `/api/summary` | Summary `?month=YYYY-MM` |
| GET | `/api/summary/trends` | Monthly income/expense trends |

## Database Schema

Auto-migrated on startup from `migrations/001_init.sql`.

- `users` - family members
- `categories` - income/expense categories with icons
- `transactions` - income/expense records (amount in cents)
- `budgets` - monthly budget per category per person

## Project Structure

```
backend/
├── Cargo.toml
├── Dockerfile
├── migrations/
│   └── 001_init.sql
└── src/
    ├── main.rs
    ├── db.rs
    ├── error.rs
    ├── models.rs
    └── handlers/
        ├── mod.rs
        ├── users.rs
        ├── categories.rs
        ├── transactions.rs
        ├── budgets.rs
        └── summary.rs
```
