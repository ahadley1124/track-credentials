# Deployment Guide

## SurrealDB Cloud Setup

The application is configured to connect to a SurrealDB cloud instance at:
- **URL:** `wss://projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud`
- **Username:** `cloud`
- **Password:** `ThisIsCloud`
- **Namespace:** `track_credentials`
- **Database:** `main`

## Database Schema

The application automatically creates the following schema on startup:

```sql
DEFINE TABLE user SCHEMAFULL;
DEFINE FIELD username ON user TYPE string;
DEFINE FIELD email ON user TYPE string;
DEFINE FIELD password_hash ON user TYPE string;
DEFINE FIELD passkey_registered ON user TYPE bool DEFAULT false;
DEFINE INDEX unique_username ON user COLUMNS username UNIQUE;
DEFINE INDEX unique_email ON user COLUMNS email UNIQUE;
```

## Local Development with SurrealDB

For local development, you can run SurrealDB locally:

1. Install SurrealDB:
```bash
# macOS or Linux
curl -sSf https://install.surrealdb.com | sh

# Or using Homebrew
brew install surrealdb/tap/surreal
```

2. Start a local SurrealDB instance:
```bash
surreal start --log trace --user root --pass root memory
```

3. Set environment variables to use local instance:
```bash
export SURREALDB_URL="ws://localhost:8000"
export SURREALDB_USER="root"
export SURREALDB_PASS="root"
```

4. Run the application:
```bash
cargo run
```

## Production Deployment

### Using Docker

Create a `Dockerfile`:

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/track-credentials /usr/local/bin/
COPY templates /app/templates
COPY static /app/static
COPY Rocket.toml /app/Rocket.toml
WORKDIR /app
EXPOSE 8000
CMD ["track-credentials"]
```

Build and run:
```bash
docker build -t track-credentials .
docker run -p 8000:8000 \
  -e SURREALDB_URL="wss://projects-06e0uks9mhrehc9sfnor9e5hbs.aws-use2.surreal.cloud" \
  -e SURREALDB_USER="cloud" \
  -e SURREALDB_PASS="ThisIsCloud" \
  track-credentials
```

### Environment Variables

- `SURREALDB_URL` - SurrealDB connection URL (default: cloud instance)
- `SURREALDB_USER` - Database username (default: "cloud")
- `SURREALDB_PASS` - Database password (default: "ThisIsCloud")
- `ROCKET_ADDRESS` - Server address (default: "0.0.0.0")
- `ROCKET_PORT` - Server port (default: 8000)

## Testing the Application

Once deployed, you can test the signup flow:

1. Navigate to `http://localhost:8000` (or your deployed URL)
2. You'll be redirected to `/signup`
3. Fill in the signup form with:
   - Username
   - Email
   - Password (minimum 8 characters)
4. Submit the form
5. You'll be redirected to `/passkey-setup`
6. Click "Complete Setup"
7. You'll be redirected to `/home`

## Verifying Database Connection

The application logs database connection status on startup. Look for:
```
Connecting to SurrealDB at: wss://...
Database initialized successfully
```

If the connection fails, check:
1. Network connectivity to the SurrealDB instance
2. Correct credentials
3. Firewall rules allowing WebSocket connections
4. DNS resolution for the cloud instance

## Security Considerations

⚠️ **Important for Production:**

1. **Password Hashing**: The current implementation uses a simple password hash. For production, implement proper password hashing with bcrypt or argon2:
   ```rust
   use bcrypt::{hash, DEFAULT_COST};
   let password_hash = hash(&form.password, DEFAULT_COST)?;
   ```

2. **Session Management**: Implement proper session management with cookies or JWT tokens.

3. **HTTPS**: Always use HTTPS in production.

4. **Environment Variables**: Never commit credentials to version control. Use environment variables or a secrets management system.

5. **Input Validation**: Add comprehensive input validation and sanitization.

6. **Rate Limiting**: Implement rate limiting to prevent abuse.

7. **WebAuthn Integration**: Complete the passkey implementation using the webauthn-rs library for actual biometric authentication.
