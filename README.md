# Track Credentials

A secure credential tracking application built with **Yew.rs** (frontend SPA) and **Rocket** (backend API), featuring WebAuthn passkey authentication and SurrealDB.

## Features

- ✅ User registration and authentication with password hashing (Argon2)
- ✅ WebAuthn passkey support as a secondary authentication method
- ✅ Yew SPA with client-side routing
- ✅ Rocket REST API backend
- ✅ SurrealDB cloud database
- ✅ Session management with encrypted cookies
- ✅ User tracking (created_at, last_login)

## Architecture

- **Frontend**: Yew.rs SPA compiled to WebAssembly, built with Trunk
- **Backend**: Rocket API server serving JSON endpoints and static files
- **Database**: SurrealDB (cloud-hosted)
- **Auth**: Password-based (primary) + WebAuthn Passkeys (secondary)

## Prerequisites

Install the following tools:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target for Yew
rustup target add wasm32-unknown-unknown

# Install Trunk (Yew build tool)
cargo install --locked trunk

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli
```

## Project Structure

```
track-credentials/
├── Cargo.toml           # Workspace configuration
├── backend/
│   ├── Cargo.toml
│   ├── Rocket.toml      # Rocket configuration
│   └── src/
│       ├── main.rs      # API routes and server setup
│       ├── db.rs        # SurrealDB connection and schema
│       └── models.rs    # Data models
└── frontend/
    ├── Cargo.toml
    ├── Trunk.toml       # Trunk build configuration
    ├── index.html       # HTML template
    ├── styles.css       # Global styles
    └── src/
        ├── main.rs      # App entry point and routing
        ├── services/    # API client functions
        ├── pages/       # Page components (Signup, Login, Home, etc.)
        └── components/  # Reusable components
```

## Database Setup

The application connects to a SurrealDB cloud instance:
- **Host**: `projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud`
- **Namespace**: `track_credentials`
- **Database**: `main`

Tables are automatically created on first run:
- **users**: Stores user accounts with username, email, password_hash, created_at, last_login
- **passkeys**: Stores WebAuthn passkey credentials

## Building and Running

### Development Mode

**Terminal 1 - Build Frontend:**
```bash
cd frontend
trunk build
```

**Terminal 2 - Run Backend:**
```bash
cd backend
cargo run
```

The backend will:
1. Connect to SurrealDB and initialize the schema
2. Start the Rocket server on `http://localhost:8000`
3. Serve the Yew app from `frontend/dist`
4. Handle all `/api/*` endpoints

**Access the app**: Open `http://localhost:8000` in your browser

### Production Build

```bash
# Build frontend (optimized)
cd frontend
trunk build --release

# Build and run backend
cd ../backend
cargo build --release
./target/release/backend
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/signup` | Register a new user |
| POST | `/api/login` | Login with username/password |
| POST | `/api/logout` | Logout current session |
| GET | `/api/user/me` | Get current user info |
| POST | `/api/passkey/register/start` | Begin passkey registration |
| POST | `/api/passkey/register/finish` | Complete passkey registration |

## User Flow

1. **Signup** (`/signup`):
   - User enters username, email, and password
   - Password is hashed with Argon2
   - User record created in SurrealDB
   - Session cookie set
   - Redirect to passkey setup

2. **Passkey Setup** (`/passkey-setup`):
   - Optional: User can register a WebAuthn passkey
   - Uses platform authenticator (fingerprint, face ID, etc.)
   - Passkey stored in database
   - Can skip and set up later
   - Redirect to home

3. **Home** (`/`):
   - Dashboard with user info
   - Placeholder cards for future features

## Security Features

- **Password Hashing**: Argon2 with random salts
- **Session Management**: Encrypted cookies (requires `ROCKET_SECRET_KEY`)
- **WebAuthn**: Industry-standard passkey authentication
- **CORS**: Configured for API access
- **Database**: Unique constraints on username/email

## Configuration

### Backend (Rocket.toml)

```toml
[default]
address = "0.0.0.0"
port = 8000

[debug]
secret_key = "hPRYyVRiMyxpw5sBB1XeCMN1kFsDCqKvBi2QJxBVHQk="

[release]
# Generate with: openssl rand -base64 32
secret_key = "<your-production-secret>"
```

### Frontend (Trunk.toml)

```toml
[[build]]
target = "index.html"
dist = "dist"

[serve]
address = "127.0.0.1"
port = 8080
```

## Development Tips

### Rebuild frontend on changes:
```bash
cd frontend
trunk watch
```

### Auto-restart backend on changes:
```bash
cd backend
cargo watch -x run
```

### Check for compilation errors:
```bash
cargo check --workspace
```

## Troubleshooting

**CORS issues**: Make sure the backend CORS configuration allows your frontend origin.

**WebAuthn not working**: Passkeys require HTTPS in production. For localhost development, HTTP is allowed.

**Database connection fails**: Verify SurrealDB credentials and network connectivity.

**Build errors**: Ensure all targets and tools are installed:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk wasm-bindgen-cli
```

## Future Enhancements

- [ ] Password reset functionality
- [ ] Email verification
- [ ] Passkey-only login flow
- [ ] Credential management features
- [ ] Activity logging
- [ ] Settings page
- [ ] Multi-device passkey support

## License

This project is licensed under the MIT License - see the LICENSE file for details.
