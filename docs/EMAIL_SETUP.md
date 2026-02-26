# Email Configuration for RustChat

## Current Status

✅ Email migrations applied  
✅ Email worker started  
⚠️  Configured with placeholder values (needs real SMTP credentials)

## Quick Setup

### Option 1: Gmail (Easiest for testing)

1. **Generate App Password** (NOT your regular Gmail password):
   - Go to https://myaccount.google.com/apppasswords
   - Select "Mail" → "Other (Custom name)" → "RustChat"
   - Copy the 16-character password

2. **Configure RustChat**:
   ```bash
   cd /Users/scolak/Projects/rustchat
   
   # Update SMTP settings
   docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
   UPDATE mail_provider_settings 
   SET 
       host = 'smtp.gmail.com',
       port = 587,
       username = 'your-email@gmail.com',
       password_encrypted = 'xxxx xxxx xxxx xxxx',  -- your app password
       from_name = 'RustChat',
       from_address = 'your-email@gmail.com',
       enabled = true
   WHERE id = '00000000-0000-0000-0000-000000000001';
   "
   ```

### Option 2: AWS SES (Production)

```bash
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
UPDATE mail_provider_settings 
SET 
    host = 'email-smtp.us-east-1.amazonaws.com',
    port = 587,
    username = 'SES-USERNAME',
    password_encrypted = 'SES-PASSWORD',
    from_name = 'RustChat',
    from_address = 'noreply@yourdomain.com',
    enabled = true
WHERE id = '00000000-0000-0000-0000-000000000001';
"
```

### Option 3: Mailgun

```bash
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
UPDATE mail_provider_settings 
SET 
    host = 'smtp.mailgun.org',
    port = 587,
    username = 'postmaster@yourdomain.com',
    password_encrypted = 'MAILGUN-PASSWORD',
    from_name = 'RustChat',
    from_address = 'noreply@yourdomain.com',
    enabled = true
WHERE id = '00000000-0000-0000-0000-000000000001';
"
```

## Test Email Configuration

After configuring, test it:

```bash
# Check logs
docker logs -f rustchat-backend 2>&1 | grep -i email

# Try password reset from the UI
# Or test via API (requires admin token):
curl -X POST http://localhost:8080/api/v4/admin/email/test \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"to_email": "your-test-email@example.com"}'
```

## Troubleshooting

### "failed to send" error in UI
- Check SMTP credentials are correct
- Check backend logs: `docker logs rustchat-backend`
- For Gmail: Make sure you're using App Password, not regular password

### Email not queued
- Check email provider is enabled: 
  ```sql
  SELECT enabled FROM mail_provider_settings;
  ```

### Check email outbox
```bash
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
SELECT status, attempt_count, recipient_email, error_message 
FROM email_outbox 
ORDER BY created_at DESC 
LIMIT 10;
"
```

## Available Email Workflows

| Workflow | Purpose |
|----------|---------|
| `user_registration` | Welcome email with verification |
| `password_reset` | Password reset link |
| `email_verification` | Email verification code |
| `offline_messages` | Missed message notifications |
| `announcements` | System announcements |

## Disable Email

If you want to disable email temporarily:

```bash
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
UPDATE mail_provider_settings SET enabled = false;
"
```
