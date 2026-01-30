# Security

## Security Features

This application implements several security best practices:

### Authentication & Authorization

- **Password Hashing**: All passwords are hashed using bcrypt with a cost factor of 12
- **WebAuthn Passkeys**: Support for FIDO2/WebAuthn passwordless authentication
- **Secure Session Management**: HTTP-only cookies with SameSite protection

### Input Validation

- **Server-side Validation**: All user inputs are validated on the server
  - Username: Minimum 3 characters
  - Email: Valid email format required
  - Password: Minimum 8 characters
- **Client-side Validation**: HTML5 form validation provides immediate feedback

### Data Protection

- **Environment Variables**: All sensitive credentials stored in environment variables
- **No Default Credentials**: Application requires explicit configuration
- **Generic Error Messages**: No internal implementation details exposed to users

### Session Security

- **HttpOnly Cookies**: Prevents XSS attacks from accessing session cookies
- **SameSite Cookies**: Provides CSRF protection
- **Session Expiration**: Registration states expire after 5 minutes
- **Automatic Cleanup**: Expired sessions are automatically cleaned up

### Database Security

- **Parameterized Queries**: SurrealDB queries use parameterized inputs
- **Namespace Isolation**: Uses dedicated namespace `track_credentials`
- **Secure Connection**: WSS (WebSocket Secure) for database communication

## Security Considerations

### Production Deployment

Before deploying to production, ensure:

1. **HTTPS**: Always use HTTPS in production
   - Update `Rocket.toml` to enable TLS
   - Update WebAuthn configuration to use HTTPS origin

2. **Environment Variables**: 
   - Never commit `.env` file
   - Use secure secrets management (AWS Secrets Manager, HashiCorp Vault, etc.)

3. **Session Storage**: 
   - Replace in-memory HashMap with Redis or similar
   - Implement proper session management with server-side tokens

4. **Rate Limiting**: 
   - Add rate limiting for signup and authentication endpoints
   - Prevent brute force attacks

5. **Cookie Security**:
   - Add `Secure` flag to cookies (requires HTTPS)
   - Consider shorter cookie expiration times

### Known Limitations

- **In-Memory Session Storage**: Registration states are stored in memory
  - Not suitable for multi-instance deployments
  - Will be lost on server restart
  - Should be replaced with Redis or similar in production

- **Cookie-based Authentication**: User ID stored in cookies
  - Consider implementing proper JWT tokens for production
  - Add token refresh mechanism

## Reporting Security Issues

If you discover a security vulnerability, please email the maintainer directly rather than opening a public issue.

## Security Checklist for Production

- [ ] Enable HTTPS/TLS
- [ ] Configure `Secure` flag on cookies
- [ ] Implement rate limiting
- [ ] Use external session store (Redis)
- [ ] Add logging and monitoring
- [ ] Regular security audits
- [ ] Keep dependencies updated
- [ ] Use secrets management service
- [ ] Configure CORS properly
- [ ] Add request size limits
