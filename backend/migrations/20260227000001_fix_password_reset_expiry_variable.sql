-- Fix password reset template - change expiry_minutes to expiry_hours
-- This aligns the template with the code and template_renderer schema

-- Fix English templates
UPDATE email_template_versions 
SET 
    body_text = REPLACE(body_text, '{{expiry_minutes}} minutes', '{{expiry_hours}} hours'),
    body_html = REPLACE(body_html, '{{expiry_minutes}} minutes', '{{expiry_hours}} hours'),
    variables_schema_json = REPLACE(
        REPLACE(variables_schema_json::text, 'expiry_minutes', 'expiry_hours'),
        '"description":"Expiry in minutes"', 
        '"description":"Hours until link expires"'
    )::jsonb
WHERE body_text LIKE '%expiry_minutes%';

-- Fix German templates
UPDATE email_template_versions 
SET 
    body_text = REPLACE(body_text, '{{expiry_minutes}} Minuten', '{{expiry_hours}} Stunden'),
    body_html = REPLACE(body_html, '{{expiry_minutes}} Minuten', '{{expiry_hours}} Stunden'),
    variables_schema_json = REPLACE(
        REPLACE(variables_schema_json::text, 'expiry_minutes', 'expiry_hours'),
        '"description":"Ablauf in Minuten"', 
        '"description":"Stunden bis zum Ablauf"'
    )::jsonb
WHERE body_text LIKE '%expiry_minutes%';
