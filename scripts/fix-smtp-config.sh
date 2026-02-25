#!/bin/bash
# Fix SMTP configuration - handles the unique constraint error

echo "=== Current SMTP Configuration ==="
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
SELECT 
    id,
    provider_type,
    host,
    port,
    username,
    enabled,
    is_default,
    from_name,
    from_address
FROM mail_provider_settings;
"

echo ""
echo "=== Options ==="
echo "1) Update existing provider (recommended)"
echo "2) Delete existing and create new"
echo "3) Exit"
echo ""
read -p "Select option (1-3): " option

if [ "$option" = "1" ]; then
    echo ""
    read -p "SMTP Host [smtp.gmail.com]: " host
    host=${host:-smtp.gmail.com}
    
    read -p "SMTP Port [587]: " port
    port=${port:-587}
    
    read -p "SMTP Username: " username
    
    read -p "SMTP Password: " -s password
    echo ""
    
    read -p "From Name [RustChat]: " from_name
    from_name=${from_name:-RustChat}
    
    read -p "From Address [$username]: " from_address
    from_address=${from_address:-$username}
    
    echo ""
    echo "Updating existing provider..."
    
    docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
    UPDATE mail_provider_settings 
    SET 
        host = '$host',
        port = $port,
        username = '$username',
        password_encrypted = '$password',
        from_name = '$from_name',
        from_address = '$from_address',
        tls_mode = 'starttls',
        enabled = true
    WHERE is_default = true;
    "
    
    echo "✅ Provider updated!"

elif [ "$option" = "2" ]; then
    echo ""
    echo "Deleting existing providers..."
    docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
    DELETE FROM mail_provider_settings;
    "
    
    echo "✅ Existing providers deleted."
    echo ""
    echo "Now you can add a new provider through the admin UI."

else
    echo "Exiting..."
    exit 0
fi

echo ""
echo "=== Verification ==="
docker exec rustchat-postgres psql -U rustchat -d rustchat -c "
SELECT 
    id,
    host,
    port,
    username,
    enabled,
    is_default,
    from_name,
    from_address
FROM mail_provider_settings
WHERE is_default = true;
"
