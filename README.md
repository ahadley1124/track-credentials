# track-credentials

A secure credential tracking application built with Rocket (Rust web framework) and SurrealDB.

## Features

- User signup with username, email, and password
- Passkey setup for enhanced security
- SurrealDB cloud integration
- Modern, responsive UI

## Prerequisites

- Rust 1.70+ (with cargo)
- Access to a SurrealDB instance

## Setup

⚠️ **Security Warning**: This is a demonstration application. Before deploying to production:
- Replace simple password hashing with bcrypt or argon2
- Implement proper session management
- Add CSRF protection
- Use HTTPS only
- Never commit credentials to version control

1. Clone the repository:
```bash
git clone https://github.com/ahadley1124/track-credentials.git
cd track-credentials
```

2. Set environment variables (required):
```bash
export SURREALDB_URL="wss://YOUR_SURREALDB_URL"
export SURREALDB_USER="YOUR_USERNAME"
export SURREALDB_PASS="YOUR_PASSWORD"
```

3. Build the project:
```bash
cargo build --release
```

4. Run the application:
```bash
cargo run --release
```

5. Open your browser and navigate to:
```
http://localhost:8000
```

## Application Flow

1. **Signup Page** (`/signup`) - Users enter username, email, and password
2. **Passkey Setup** (`/passkey-setup`) - Users set up passkey authentication
3. **Home Page** (`/home`) - User dashboard after successful signup

## Database Schema

The application uses SurrealDB with the following schema:

**Namespace:** `track_credentials`
**Database:** `main`

**User Table:**
- `username` (string, unique)
- `email` (string, unique)
- `password_hash` (string)
- `passkey_registered` (boolean, default: false)

## Development

The application is built using:
- **Rocket 0.5** - Fast, type-safe web framework
- **SurrealDB 1.1** - Modern, distributed database
- **Tera** - Template engine for HTML rendering
- **WebAuthn-rs** - Passkey/WebAuthn support (future integration)

## Project Structure

```
track-credentials/
├── src/
│   ├── main.rs          # Application entry point and routes
│   └── db.rs            # Database connection and initialization
├── templates/           # HTML templates
│   ├── signup.html.tera
│   ├── passkey_setup.html.tera
│   └── home.html.tera
├── static/              # Static assets
│   └── style.css
└── Cargo.toml          # Project dependencies
```

## License

See LICENSE file for details.
