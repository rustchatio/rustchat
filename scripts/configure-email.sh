#!/bin/bash
# Configure email for RustChat
# Usage: ./configure-email.sh [smtp-host] [port] [username] [password] [sender-name] [sender-email]

set -e

# Default values
SMTP_HOST=${1:-"smtp.gmail.com"}
SMTP_PORT=${2:-587}
SMTP_USER=${3:-""}
SMTP_PASS=${4:-""}
SENDER_NAME=${5:-"RustChat"}
SENDER_EMAIL=${6:-""}

if [ -z "$SMTP_USER" ] || [ -z "$SMTP_PASS" ]; then
    echo "Usage: $0 [smtp-host] [port] [username] [password] [sender-name] [sender-email]"
    echo ""
    echo "Examples:"
    echo "  # Gmail:"
    echo "  $0 smtp.gmail.com 587 your-email@gmail.com 'app-password' 'RustChat' 'noreply@yourdomain.com'"
    echo ""
    echo "  # AWS SES:"
    echo "  $0 email-smtp.us-east-1.amazonaws.com 587 'SES-USERNAME' 'SES-PASSWORD' 'RustChat' 'noreply@yourdomain.com'"
    echo ""
    echo "  # Mailgun:"
    echo "  $0 smtp.mailgun.org 587 'postmaster@yourdomain.com' 'mailgun-password' 'RustChat' 'noreply@yourdomain.com'"
    echo ""
    echo "Note: For Gmail, use an App Password (not your regular password):"
    echo "  https://support.google.com/accounts/answer/185833"
    exit 1
fi

echo "=== Configuring Email for RustChat ==="
echo "SMTP Host: $SMTP_HOST"
echo "SMTP Port: $SMTP_PORT"
echo "Username: $SMTP_USER"
echo "Sender: $SENDER_NAME <$SENDER_EMAIL>"
echo ""

# Update the mail provider settings
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
UPDATE mail_provider_settings 
SET 
    enabled = true,
    host = '$SMTP_HOST',
    port = $SMTP_PORT,
    username = '$SMTP_USER',
    password_encrypted = '$SMTP_PASS',
    from_name = '$SENDER_NAME',
    from_address = '$SENDER_EMAIL',
    tls_mode = 'starttls'
WHERE id = '00000000-0000-0000-0000-000000000001';
"

echo ""
echo "=== Testing Email Configuration ==="

# Test the connection using the backend API
# This requires admin authentication
RESPONSE=$(curl -s -X POST http://localhost:8080/api/v4/admin/email/test \
  -H "Content-Type: application/json" \
  -d '{"provider_id": "00000000-0000-0000-0000-000000000001"}' \
  2>/dev/null || echo '{"success": false, "message": "Connection failed"}')

echo "Test response: $RESPONSE"
echo ""
echo "=== Configuration Complete ==="
echo ""
echo "Next steps:"
echo "1. Test password reset from the login page"
echo "2. Test new user registration email"
echo "3. Check backend logs: docker logs -f rustchat-backend"
