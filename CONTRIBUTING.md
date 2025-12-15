# Contributing to Booking Headless API

Welcome! We are thrilled that you’re interested in contributing.

This project is a **Headless, Fail-Fast Appointment Scheduling Engine**. We aim to build a backend that is resilient, strictly typed, and client-agnostic—meaning it works just as well for a CLI tool as it does for a React Native app.

This document serves as your map to the codebase. It covers our philosophy, setup, testing strategies, and submission guidelines.

---

## 🧭 Project Philosophy

To contribute effectively, it helps to understand the core principles driving this architecture:

1.  **Fail Fast:** We prefer the application to crash at startup rather than run with a broken configuration. If an environment variable is missing or a DB connection fails, the app panics immediately.
2.  **Headless & Agnostic:** We do not serve HTML. We serve JSON. All logic (timezones, error messages, availability) must be usable by _any_ frontend.
3.  **Time is Absolute:** All times are stored and processed in **UTC**. Conversion to "Local Time" happens only at the very edges of the application (input/output).
4.  **Separation of Concerns:**
    - **Routes:** Handle HTTP requests/responses only.
    - **Utils:** Contain pure business logic (math, date calculation).
    - **Structs:** Define the data contracts (DB models, JSON bodies).

---

## 🛠 Prerequisites

Ensure you have the following installed before starting:

- **Rust (Latest Stable):** [Install Rust](https://www.rust-lang.org/tools/install)
- **Docker & Docker Compose:** For running the database and cache.
- **SQLx CLI:** Essential for database management.
  ```bash
  cargo install sqlx-cli
  ```

---

## 🚀 Development Environment Setup

### 1. Fork and Clone

Fork the repository to your GitHub account, then clone it locally:

```bash
git clone [https://github.com/onfranciis/booking-headless.git](https://github.com/onfranciis/booking-headless.git)
cd booking-headless
```

### 2\. Configure Environment

We utilize a strict `.env` configuration.

```bash
# Create your local env file from the template
cp .env.example .env
```

To ensure your commits always pass CI, please install the git hook:

```bash
cp scripts/pre-commit.sh .git/hooks/pre-commit
```

> **Important:** You must fill in `GOOGLE_CLIENT_ID` and `GOOGLE_CLIENT_SECRET` in `.env` if you intend to work on Authentication features.

### 3\. Start Infrastructure

Spin up the PostgreSQL database and Redis cache:

```bash
docker-compose up -d db redis
```

### 4\. Database Initialization

Run the migrations to create the schema and tables:

```bash
sqlx migrate run
```

### 5\. Run the Server

```bash
cargo run
```

The API is now live at `http://127.0.0.1:8080`.

---

## 📂 Architecture Guide

Knowing where to put code is half the battle.

| Directory          | Purpose                                                                                                                      |
| :----------------- | :--------------------------------------------------------------------------------------------------------------------------- |
| `src/routes/`      | **Controllers.** Contains the endpoint handlers (e.g., `user_routes.rs`). Keep logic here minimal. Delegate math to `utils`. |
| `src/utils/`       | **Pure Logic.** Helper functions that are side-effect free. Example: `generate_slots` (Time math).                           |
| `src/structs/`     | **Types.** Database models (`db_struct.rs`) and API request/response schemas (`util_struct.rs`).                             |
| `src/middlewares/` | **Interceptors.** Authentication checks and request processing.                                                              |
| `src/tests/`       | **Unit Tests.** Dedicated folder for testing pure logic without spinning up the server.                                      |
| `migrations/`      | **SQL.** Raw SQL files for database schema changes.                                                                          |

---

## 🧪 Testing Strategy

We maintain a high standard for reliability. **All PRs must pass tests.**

### 1\. Logic Tests (Unit)

These test the "Pure Math" of the engine (e.g., "Does a booking at 11 PM overlap with a shift ending at 2 AM?"). These are fast and require no database.

```bash
cargo test
```

### 2\. API Contract Tests (Manual)

We use **Swagger/OpenAPI** to verify endpoints.

1.  Run the app: `cargo run`
2.  Visit: `http://localhost:8080/swagger-ui/`
3.  Execute requests against your local server to ensure JSON structures match the documentation.

---

## 💾 Database Workflow (SQLx)

We use **SQLx** for compile-time checked queries. This requires a specific workflow when changing SQL.

**If you modify a SQL query or add a migration:**

1.  Ensure your local DB is running.
2.  Run the query locally (via `cargo run` or tests).
3.  **Update the offline metadata file.** This is required for the CI/Docker build to pass without a live database connection.
    ```bash
    cargo sqlx prepare -- --lib
    ```

---

## 🤝 Submission Guidelines

### Commit Messages

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This allows us to generate changelogs automatically.

- `feat: add google calendar sync`
- `fix: resolve timezone overlap bug`
- `docs: update readme architecture diagram`
- `refactor: extract slot generation logic`

### The Pull Request Process

1.  Create a new branch: `git checkout -b feat/my-new-feature`.
2.  Write your code.
3.  **Add Tests:** If you added logic, add a test case in `src/tests/unit_test.rs`.
4.  Format code: `cargo fmt`.
5.  Lint code: `cargo clippy`.
6.  Push and open a Pull Request.

---

## ❓ Troubleshooting

**Q: `sqlx-data.json` errors during build?**
A: You likely changed a query but forgot to run `cargo sqlx prepare -- --lib`. Run that command and commit the changes.

**Q: "Google Auth Failed"?**\
A: Ensure your `.env` file has valid credentials and that your Google Cloud Console "Redirect URI" matches `http://localhost:8080/auth/callback`.

**Q: Docker build fails?**\
A: Check if you included the `sqlx-data.json` file in your commit. The Dockerfile uses offline mode and needs this file.

---

Thank you for building with us\! 🦀
