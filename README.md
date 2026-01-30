# track-credentials

A secure credential tracking application built with Rocket (Rust web framework), SurrealDB, and WebAuthn passkey support.

## Features

- **User Registration**: Sign up with username, email, and password
- **Passkey Support**: Secure authentication using WebAuthn passkeys
- **SurrealDB Backend**: Cloud-based database for secure data storage
- **Modern UI**: Beautiful, responsive signup and authentication flow

## Prerequisites

- Rust 1.70 or higher
- SurrealDB cloud instance access

## Setup

1. Clone the repository:
```bash
git clone https://github.com/ahadley1124/track-credentials.git
cd track-credentials
```

2. Configure database credentials:
   - Copy `.env.example` to `.env`
   - Update the SurrealDB credentials in `.env`:
     ```env
     SURREAL_URL=your-project.aws-use2.surreal.cloud
     SURREAL_USER=your_username
     SURREAL_PASS=your_password
     ```
   - **Important**: All three environment variables are required for the application to run

3. Build and run:
```bash
cargo run
```

4. Open your browser to: `http://localhost:8000`

## Database Structure

The application automatically creates:
- **Namespace**: `track_credentials`
- **Database**: `main`
- **Table**: `user` with fields:
  - `username`: String
  - `email`: String
  - `password_hash`: String (bcrypt hashed)
  - `passkey`: Optional array of WebAuthn passkeys

## User Flow

1. **Signup** (`/signup`): Enter username, email, and password
2. **Setup Passkey** (`/setup-passkey`): Configure WebAuthn passkey (optional)
3. **Home** (`/home`): Access authenticated dashboard

## Security Features

- Password hashing with bcrypt
- WebAuthn passkey support for passwordless authentication
- HTTPS-ready configuration
- CSRF protection via Rocket's Shield

## Development

Run in development mode:
```bash
cargo run
```

Build for production:
```bash
cargo build --release
```

## License

See LICENSE file for details.
