# Track Credentials

A secure credential tracking application with WebAuthn passkey support.

## Architecture

- **Backend**: Rust + Rocket web framework (API server)
- **Frontend**: Rust + Yew (WebAssembly SPA)
- **Database**: SurrealDB (cloud-hosted)
- **Authentication**: Password + WebAuthn Passkeys

## Features

- ✅ User signup with email and password
- ✅ Password-based authentication with Argon2 hashing
- ✅ WebAuthn passkey support (biometric authentication)
- ✅ Session management with encrypted cookies
- ✅ SPA with client-side routing
- ✅ Modern, responsive UI

## Project Structure

```
track-credentials/
├── backend/          # Rocket API server
│   ├── src/
│   │   ├── main.rs   # API routes and server setup
│   │   ├── db.rs     # SurrealDB connection and schema
│   │   └── models.rs # Data models
│   └── Cargo.toml
├── frontend/         # Yew WebAssembly app
│   ├── src/
│   │   ├── main.rs   # App entry point and routing
│   │   ├── pages/    # Page components
│   │   └── services/ # API client
│   ├── index.html
│   ├── styles.css
│   └── Cargo.toml
└── Cargo.toml        # Workspace manifest
```

## Prerequisites

- Rust (latest stable)
- Trunk (for building the frontend): `cargo install trunk`
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`

## Database Setup

The application connects to a SurrealDB cloud instance:
- **Host**: projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud
- **Namespace**: track_credentials
- **Database**: main

Tables are automatically created on first run:
- `users` - User accounts with username, email, password hash
- `passkeys` - WebAuthn passkey credentials

## Building

### Quick Build (Both Frontend and Backend)

```bash
./build.sh
```

### Manual Build

**Frontend:**
```bash
cd frontend
trunk build
```

**Backend:**
```bash
cd backend
cargo build
```

## Running

### Development Mode

```bash
cd backend
cargo run
```

The server will start on `http://localhost:8000`

### Production Mode

```bash
cd frontend
trunk build --release

cd ../backend
cargo build --release
./target/release/backend
```

## Usage

1. **Navigate to** `http://localhost:8000`
2. **Sign Up**: Create an account with username, email, and password
3. **Setup Passkey** (optional): Configure biometric authentication
4. **Home Dashboard**: Access your credential tracking dashboard

## API Endpoints

### Authentication
- `POST /api/signup` - Create new user account
- `POST /api/login` - Authenticate with username/password
- `POST /api/logout` - End session
- `GET /api/user/me` - Get current user info

### Passkeys
- `POST /api/passkey/register/start` - Begin passkey registration
- `POST /api/passkey/register/finish` - Complete passkey registration

## Development

### Frontend Development

For live reloading during development:
```bash
cd frontend
trunk serve
```

Then access the frontend at `http://localhost:8080` (it will proxy API requests to the backend on port 8000).

### Backend Development

```bash
cd backend
cargo watch -x run
```

## Security Features

- **Password Hashing**: Argon2 with salting
- **WebAuthn/FIDO2**: Passkey support for passwordless authentication
- **Encrypted Cookies**: Session management with Rocket's private cookies
- **HTTPS Ready**: Configure TLS in production
- **CORS Protection**: Configurable CORS policies

## Browser Compatibility

Passkey/WebAuthn support requires:
- Chrome/Edge 67+
- Firefox 60+
- Safari 13+
- Opera 54+

## Troubleshooting

### Frontend doesn't load
- Ensure `trunk build` completed successfully
- Check that `frontend/dist/` directory exists
- Verify backend is serving static files from the correct path

### Database connection errors
- Verify SurrealDB credentials
- Check network connectivity
- Ensure namespace and database are created

### Passkey registration fails
- Use HTTPS in production (WebAuthn requires secure context)
- For localhost development, HTTP is allowed
- Verify browser supports WebAuthn

## License

See LICENSE file for details.
