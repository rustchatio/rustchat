#!/bin/bash
#
# RustChat Environment Generator
#
# This script generates a secure .env file with cryptographically
# strong random secrets. Use this for initial setup or secret rotation.
#
# Usage: ./tools/generate-env.sh
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$PROJECT_ROOT/.env"
ENV_EXAMPLE="$PROJECT_ROOT/.env.example"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Banner
echo -e "${BLUE}"
echo "╔══════════════════════════════════════════════════════════════════╗"
echo "║           RustChat Environment Generator                         ║"
echo "║                                                                  ║"
echo "║  This script will generate secure random secrets for your        ║"
echo "║  RustChat installation and create a .env file.                   ║"
echo "╚══════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"
echo ""

# Check if .env already exists
if [ -f "$ENV_FILE" ]; then
    echo -e "${YELLOW}⚠️  WARNING: .env file already exists at:${NC}"
    echo "   $ENV_FILE"
    echo ""
    echo -e "${RED}This operation will DELETE and RECREATE your .env file!${NC}"
    echo "Any custom configurations you've made will be lost."
    echo ""
    echo "The following secrets will be regenerated:"
    echo "  • RUSTCHAT_JWT_SECRET (48 bytes base64)"
    echo "  • RUSTCHAT_JWT_ISSUER (unique issuer ID)"
    echo "  • RUSTCHAT_JWT_AUDIENCE (audience)"
    echo "  • RUSTCHAT_ENCRYPTION_KEY (48 bytes base64)"
    echo "  • RUSTCHAT_S3_ACCESS_KEY / RUSTFS_ACCESS_KEY (32 bytes hex, shared)"
    echo "  • RUSTCHAT_S3_SECRET_KEY / RUSTFS_SECRET_KEY (64 bytes base64, shared)"
    echo "  • TURN_SERVER_USERNAME (32 bytes hex)"
    echo "  • TURN_SERVER_CREDENTIAL (48 bytes base64)"
    echo ""
    read -p "Are you sure you want to continue? Type 'yes' to proceed: " CONFIRM
    
    if [ "$CONFIRM" != "yes" ]; then
        echo ""
        echo -e "${YELLOW}Operation cancelled. Your .env file was not modified.${NC}"
        exit 0
    fi
    
    # Backup existing .env
    BACKUP_FILE="$ENV_FILE.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$ENV_FILE" "$BACKUP_FILE"
    echo ""
    echo -e "${GREEN}✓ Created backup: $BACKUP_FILE${NC}"
fi

# Check if .env.example exists
if [ ! -f "$ENV_EXAMPLE" ]; then
    echo -e "${RED}Error: .env.example not found at $ENV_EXAMPLE${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}Generating secure random secrets...${NC}"
echo ""

# Function to generate base64 secret
generate_base64_secret() {
    local length=$1
    openssl rand -base64 "$length" | tr -d '\n'
}

# Function to generate hex secret
generate_hex_secret() {
    local length=$1
    openssl rand -hex "$length" | tr -d '\n'
}

# Generate secrets
JWT_SECRET=$(generate_base64_secret 48)
JWT_ISSUER="rustchat-$(generate_hex_secret 8)"
JWT_AUDIENCE="rustchat-users"
ENCRYPTION_KEY=$(generate_base64_secret 48)
S3_ACCESS_KEY=$(generate_hex_secret 16)
S3_SECRET_KEY=$(generate_base64_secret 48)
# Backend and RustFS must use the same credentials when docker-compose
# points backend S3 endpoint to the bundled rustfs service.
RUSTFS_ACCESS_KEY="$S3_ACCESS_KEY"
RUSTFS_SECRET_KEY="$S3_SECRET_KEY"
TURN_USERNAME=$(generate_hex_secret 16)
TURN_CREDENTIAL=$(generate_base64_secret 48)

echo -e "  ${GREEN}✓${NC} JWT_SECRET generated (48 bytes base64)"
echo -e "  ${GREEN}✓${NC} JWT_ISSUER generated (unique issuer)"
echo -e "  ${GREEN}✓${NC} JWT_AUDIENCE generated (audience)"
echo -e "  ${GREEN}✓${NC} ENCRYPTION_KEY generated (48 bytes base64)"
echo -e "  ${GREEN}✓${NC} S3_ACCESS_KEY generated (32 hex chars, shared with RustFS)"
echo -e "  ${GREEN}✓${NC} S3_SECRET_KEY generated (48 bytes base64, shared with RustFS)"
echo -e "  ${GREEN}✓${NC} TURN_SERVER_USERNAME generated (32 hex chars)"
echo -e "  ${GREEN}✓${NC} TURN_SERVER_CREDENTIAL generated (48 bytes base64)"
echo ""

# Create .env file from template
echo -e "${BLUE}Creating .env file...${NC}"

# Read .env.example and substitute secrets
sed -e "s|^RUSTCHAT_JWT_SECRET=.*|RUSTCHAT_JWT_SECRET=$JWT_SECRET|" \
    -e "s|^RUSTCHAT_JWT_ISSUER=.*|RUSTCHAT_JWT_ISSUER=$JWT_ISSUER|" \
    -e "s|^RUSTCHAT_JWT_AUDIENCE=.*|RUSTCHAT_JWT_AUDIENCE=$JWT_AUDIENCE|" \
    -e "s|^RUSTCHAT_ENCRYPTION_KEY=.*|RUSTCHAT_ENCRYPTION_KEY=$ENCRYPTION_KEY|" \
    -e "s|^RUSTCHAT_S3_ACCESS_KEY=.*|RUSTCHAT_S3_ACCESS_KEY=$S3_ACCESS_KEY|" \
    -e "s|^RUSTCHAT_S3_SECRET_KEY=.*|RUSTCHAT_S3_SECRET_KEY=$S3_SECRET_KEY|" \
    -e "s|^RUSTFS_ACCESS_KEY=.*|RUSTFS_ACCESS_KEY=$RUSTFS_ACCESS_KEY|" \
    -e "s|^RUSTFS_SECRET_KEY=.*|RUSTFS_SECRET_KEY=$RUSTFS_SECRET_KEY|" \
    -e "s|^TURN_SERVER_USERNAME=.*|TURN_SERVER_USERNAME=$TURN_USERNAME|" \
    -e "s|^TURN_SERVER_CREDENTIAL=.*|TURN_SERVER_CREDENTIAL=$TURN_CREDENTIAL|" \
    "$ENV_EXAMPLE" > "$ENV_FILE"

# Set secure permissions (readable only by owner)
chmod 600 "$ENV_FILE"

echo ""
echo -e "${GREEN}✓ .env file created successfully at:${NC}"
echo "   $ENV_FILE"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "  1. Review the generated .env file and adjust as needed:"
echo "     nano $ENV_FILE"
echo ""
echo "  2. For production deployment, make sure to:"
echo "     • Set RUSTCHAT_ENVIRONMENT=production"
echo "     • Configure RUSTCHAT_SITE_URL to your public URL (must be https://)"
echo "     • Update RUSTCHAT_CORS_ALLOWED_ORIGINS to https:// origins ONLY"
echo "     • Update database and Redis connection strings"
echo "     • Configure S3 endpoint and bucket settings"
echo "     • Review security settings at the bottom of the file"
echo ""
echo "  3. Start RustChat:"
echo "     docker-compose up -d"
echo ""
echo -e "${YELLOW}⚠️  Security Notice:${NC}"
echo "   Keep your .env file secure and never commit it to version control."
echo "   The secrets generated are cryptographically strong and unique."
echo ""
