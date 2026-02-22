-- Seed default email templates
-- Creates published v1 templates in English and German for the built-in workflows

-- ============================================
-- Registration Verification Template
-- ============================================

-- English version
INSERT INTO email_template_versions (
    id, family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by
) VALUES (
    '00000000-0000-0000-0000-000000000201'::uuid,
    '00000000-0000-0000-0000-000000000101'::uuid,
    1,
    'published',
    'en',
    'Welcome to {{site_name}} - Verify Your Email',
    'Hi {{user_name}},

Welcome to {{site_name}}! Please verify your email address by clicking the link below:

{{verification_link}}

This link will expire in 24 hours.

If you did not create an account, you can safely ignore this email.

Best regards,
The {{site_name}} Team

---
{{site_url}}',
    '<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Welcome to {{site_name}}</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #4f46e5; color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border-radius: 0 0 8px 8px; }
        .button { display: inline-block; background: #4f46e5; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 20px 0; }
        .footer { margin-top: 30px; font-size: 12px; color: #6b7280; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Welcome to {{site_name}}!</h1>
    </div>
    <div class="content">
        <p>Hi {{user_name}},</p>
        <p>Thank you for joining {{site_name}}. Please verify your email address by clicking the button below:</p>
        <p style="text-align: center;">
            <a href="{{verification_link}}" class="button">Verify Email Address</a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #4f46e5;">{{verification_link}}</p>
        <p>This link will expire in 24 hours.</p>
        <p>If you did not create an account, you can safely ignore this email.</p>
        <div class="footer">
            <p>Best regards,<br>The {{site_name}} Team</p>
            <p><a href="{{site_url}}">{{site_url}}</a></p>
        </div>
    </div>
</body>
</html>',
    '[
        {"name": "user_name", "required": true, "description": "The user''s display name"},
        {"name": "email", "required": true, "description": "The user''s email address"},
        {"name": "verification_link", "required": true, "description": "Link to verify email address"},
        {"name": "site_name", "required": false, "default_value": "RustChat", "description": "The site name"},
        {"name": "site_url", "required": false, "description": "The site URL"}
    ]'::jsonb,
    false,
    NULL
) ON CONFLICT (family_id, version, locale) DO NOTHING;

-- German version
INSERT INTO email_template_versions (
    id, family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by
) VALUES (
    '00000000-0000-0000-0000-000000000202'::uuid,
    '00000000-0000-0000-0000-000000000101'::uuid,
    1,
    'published',
    'de',
    'Willkommen bei {{site_name}} - E-Mail bestätigen',
    'Hallo {{user_name}},

Willkommen bei {{site_name}}! Bitte bestätigen Sie Ihre E-Mail-Adresse, indem Sie auf den folgenden Link klicken:

{{verification_link}}

Dieser Link läuft in 24 Stunden ab.

Wenn Sie kein Konto erstellt haben, können Sie diese E-Mail ignorieren.

Mit freundlichen Grüßen,
Das {{site_name}} Team

---
{{site_url}}',
    '<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Willkommen bei {{site_name}}</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #4f46e5; color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border-radius: 0 0 8px 8px; }
        .button { display: inline-block; background: #4f46e5; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 20px 0; }
        .footer { margin-top: 30px; font-size: 12px; color: #6b7280; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Willkommen bei {{site_name}}!</h1>
    </div>
    <div class="content">
        <p>Hallo {{user_name}},</p>
        <p>Vielen Dank für Ihre Registrierung bei {{site_name}}. Bitte bestätigen Sie Ihre E-Mail-Adresse:</p>
        <p style="text-align: center;">
            <a href="{{verification_link}}" class="button">E-Mail bestätigen</a>
        </p>
        <p>Oder kopieren Sie diesen Link in Ihren Browser:</p>
        <p style="word-break: break-all; color: #4f46e5;">{{verification_link}}</p>
        <p>Dieser Link läuft in 24 Stunden ab.</p>
        <p>Wenn Sie kein Konto erstellt haben, können Sie diese E-Mail ignorieren.</p>
        <div class="footer">
            <p>Mit freundlichen Grüßen,<br>Das {{site_name}} Team</p>
            <p><a href="{{site_url}}">{{site_url}}</a></p>
        </div>
    </div>
</body>
</html>',
    '[
        {"name": "user_name", "required": true, "description": "Anzeigename des Benutzers"},
        {"name": "email", "required": true, "description": "E-Mail-Adresse des Benutzers"},
        {"name": "verification_link", "required": true, "description": "Link zur E-Mail-Bestätigung"},
        {"name": "site_name", "required": false, "default_value": "RustChat", "description": "Name der Website"},
        {"name": "site_url", "required": false, "description": "URL der Website"}
    ]'::jsonb,
    false,
    NULL
) ON CONFLICT (family_id, version, locale) DO NOTHING;

-- ============================================
-- Password Reset Template
-- ============================================

-- English version
INSERT INTO email_template_versions (
    id, family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by
) VALUES (
    '00000000-0000-0000-0000-000000000203'::uuid,
    '00000000-0000-0000-0000-000000000102'::uuid,
    1,
    'published',
    'en',
    'Password Reset Request - {{site_name}}',
    'Hi {{user_name}},

We received a request to reset your password for your {{site_name}} account.

Click the link below to reset your password:

{{reset_link}}

This link will expire in {{expiry_hours}} hours.

If you did not request a password reset, please ignore this email or contact support if you have concerns.

Best regards,
The {{site_name}} Team

---
For security, this request was received from a device. If this was not you, please secure your account immediately.',
    '<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Password Reset Request</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #dc2626; color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border-radius: 0 0 8px 8px; }
        .button { display: inline-block; background: #dc2626; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 20px 0; }
        .footer { margin-top: 30px; font-size: 12px; color: #6b7280; }
        .security-note { background: #fef3c7; border-left: 4px solid #f59e0b; padding: 15px; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Password Reset Request</h1>
    </div>
    <div class="content">
        <p>Hi {{user_name}},</p>
        <p>We received a request to reset your password for your {{site_name}} account.</p>
        <p style="text-align: center;">
            <a href="{{reset_link}}" class="button">Reset Password</a>
        </p>
        <p>Or copy and paste this link into your browser:</p>
        <p style="word-break: break-all; color: #dc2626;">{{reset_link}}</p>
        <p>This link will expire in {{expiry_hours}} hours.</p>
        <div class="security-note">
            <strong>Didn''t request this?</strong>
            <p>If you did not request a password reset, please ignore this email. Your password will not be changed.</p>
        </div>
        <div class="footer">
            <p>Best regards,<br>The {{site_name}} Team</p>
        </div>
    </div>
</body>
</html>',
    '[
        {"name": "user_name", "required": true, "description": "The user''s display name"},
        {"name": "reset_link", "required": true, "description": "Link to reset password"},
        {"name": "expiry_hours", "required": false, "default_value": "24", "description": "Hours until link expires"},
        {"name": "site_name", "required": false, "default_value": "RustChat", "description": "The site name"}
    ]'::jsonb,
    false,
    NULL
) ON CONFLICT (family_id, version, locale) DO NOTHING;

-- German version
INSERT INTO email_template_versions (
    id, family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by
) VALUES (
    '00000000-0000-0000-0000-000000000204'::uuid,
    '00000000-0000-0000-0000-000000000102'::uuid,
    1,
    'published',
    'de',
    'Passwort zurücksetzen - {{site_name}}',
    'Hallo {{user_name}},

Wir haben eine Anfrage zum Zurücksetzen Ihres Passworts für Ihr {{site_name}}-Konto erhalten.

Klicken Sie auf den folgenden Link, um Ihr Passwort zurückzusetzen:

{{reset_link}}

Dieser Link läuft in {{expiry_hours}} Stunden ab.

Wenn Sie keine Passwortzurücksetzung angefordert haben, ignorieren Sie diese E-Mail bitte.

Mit freundlichen Grüßen,
Das {{site_name}} Team',
    '<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Passwort zurücksetzen</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #dc2626; color: white; padding: 30px; text-align: center; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border-radius: 0 0 8px 8px; }
        .button { display: inline-block; background: #dc2626; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 20px 0; }
        .footer { margin-top: 30px; font-size: 12px; color: #6b7280; }
        .security-note { background: #fef3c7; border-left: 4px solid #f59e0b; padding: 15px; margin: 20px 0; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Passwort zurücksetzen</h1>
    </div>
    <div class="content">
        <p>Hallo {{user_name}},</p>
        <p>Wir haben eine Anfrage zum Zurücksetzen Ihres Passworts erhalten.</p>
        <p style="text-align: center;">
            <a href="{{reset_link}}" class="button">Passwort zurücksetzen</a>
        </p>
        <p>Oder kopieren Sie diesen Link in Ihren Browser:</p>
        <p style="word-break: break-all; color: #dc2626;">{{reset_link}}</p>
        <p>Dieser Link läuft in {{expiry_hours}} Stunden ab.</p>
        <div class="security-note">
            <strong>Nicht angefordert?</strong>
            <p>Wenn Sie keine Passwortzurücksetzung angefordert haben, ignorieren Sie diese E-Mail.</p>
        </div>
        <div class="footer">
            <p>Mit freundlichen Grüßen,<br>Das {{site_name}} Team</p>
        </div>
    </div>
</body>
</html>',
    '[
        {"name": "user_name", "required": true, "description": "Anzeigename des Benutzers"},
        {"name": "reset_link", "required": true, "description": "Link zum Zurücksetzen des Passworts"},
        {"name": "expiry_hours", "required": false, "default_value": "24", "description": "Stunden bis zum Ablauf des Links"},
        {"name": "site_name", "required": false, "default_value": "RustChat", "description": "Name der Website"}
    ]'::jsonb,
    false,
    NULL
) ON CONFLICT (family_id, version, locale) DO NOTHING;

-- ============================================
-- Offline Messages Template
-- ============================================

-- English version
INSERT INTO email_template_versions (
    id, family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by
) VALUES (
    '00000000-0000-0000-0000-000000000205'::uuid,
    '00000000-0000-0000-0000-000000000104'::uuid,
    1,
    'published',
    'en',
    '{{message_count}} new messages in #{{channel_name}}',
    'Hi {{user_name}},

You have {{message_count}} new messages in #{{channel_name}} {{#if team_name}}in {{team_name}}{{/if}}.

{{#if message_excerpt}}Latest message from {{sender_name}}:
"{{message_excerpt}}"

{{/if}}Click here to view: {{channel_link}}

---
You received this because you enabled offline notifications. To change your settings, visit: {{site_url}}/preferences/notifications',
    '<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>New Messages</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #4f46e5; color: white; padding: 20px 30px; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border-radius: 0 0 8px 8px; }
        .message-preview { background: white; border-left: 4px solid #4f46e5; padding: 15px; margin: 20px 0; }
        .button { display: inline-block; background: #4f46e5; color: white; padding: 12px 24px; text-decoration: none; border-radius: 6px; margin: 20px 0; }
        .footer { margin-top: 30px; font-size: 12px; color: #6b7280; border-top: 1px solid #e5e7eb; padding-top: 20px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>{{message_count}} new messages</h1>
    </div>
    <div class="content">
        <p>Hi {{user_name}},</p>
        <p>You have <strong>{{message_count}} new messages</strong> in #{{channel_name}} {{#if team_name}}in {{team_name}}{{/if}}.</p>
        {{#if message_excerpt}}
        <div class="message-preview">
            <p><strong>{{sender_name}}:</strong></p>
            <p>"{{message_excerpt}}"</p>
        </div>
        {{/if}}
        <p>
            <a href="{{channel_link}}" class="button">View Messages</a>
        </p>
        <div class="footer">
            <p>You received this because you enabled offline notifications.</p>
            <p><a href="{{site_url}}/preferences/notifications">Change notification settings</a></p>
        </div>
    </div>
</body>
</html>',
    '[
        {"name": "user_name", "required": true, "description": "The recipient''s name"},
        {"name": "channel_name", "required": true, "description": "The channel name"},
        {"name": "team_name", "required": false, "description": "The team name"},
        {"name": "message_count", "required": true, "description": "Number of unread messages"},
        {"name": "message_excerpt", "required": false, "description": "Preview of the latest message"},
        {"name": "sender_name", "required": false, "description": "Name of the sender"},
        {"name": "channel_link", "required": false, "description": "Direct link to the channel"},
        {"name": "site_url", "required": false, "description": "Site URL for preferences link"}
    ]'::jsonb,
    false,
    NULL
) ON CONFLICT (family_id, version, locale) DO NOTHING;

-- ============================================
-- Announcements Template
-- ============================================

-- English version
INSERT INTO email_template_versions (
    id, family_id, version, status, locale, subject, body_text, body_html,
    variables_schema_json, is_compiled_from_mjml, created_by
) VALUES (
    '00000000-0000-0000-0000-000000000207'::uuid,
    '00000000-0000-0000-0000-000000000103'::uuid,
    1,
    'published',
    'en',
    '{{subject}}',
    'Hi {{#if user_name}}{{user_name}}{{else}}there{{/if}},

{{content}}

---
This announcement was sent by {{from_name}} on behalf of {{site_name}}.
{{#if unsubscribe_link}}
To unsubscribe from announcements: {{unsubscribe_link}}
{{/if}}',
    '<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{{subject}}</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px; }
        .header { background: #4f46e5; color: white; padding: 20px 30px; border-radius: 8px 8px 0 0; }
        .content { background: #f9fafb; padding: 30px; border-radius: 0 0 8px 8px; }
        .announcement { background: white; padding: 20px; margin: 20px 0; border-radius: 6px; }
        .footer { margin-top: 30px; font-size: 12px; color: #6b7280; border-top: 1px solid #e5e7eb; padding-top: 20px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>{{site_name}} Announcement</h1>
    </div>
    <div class="content">
        <p>Hi {{#if user_name}}{{user_name}}{{else}}there{{/if}},</p>
        <div class="announcement">
            <h2>{{subject}}</h2>
            <p>{{content}}</p>
        </div>
        <div class="footer">
            <p>This announcement was sent by {{from_name}} on behalf of {{site_name}}.</p>
            {{#if unsubscribe_link}}
            <p><a href="{{unsubscribe_link}}">Unsubscribe from announcements</a></p>
            {{/if}}
        </div>
    </div>
</body>
</html>',
    '[
        {"name": "user_name", "required": false, "description": "The recipient''s name"},
        {"name": "subject", "required": true, "description": "The announcement subject"},
        {"name": "content", "required": true, "description": "The announcement content"},
        {"name": "from_name", "required": false, "description": "Name of the sender/admin"},
        {"name": "site_name", "required": false, "default_value": "RustChat", "description": "The site name"},
        {"name": "unsubscribe_link", "required": false, "description": "Link to unsubscribe from announcements"}
    ]'::jsonb,
    false,
    NULL
) ON CONFLICT (family_id, version, locale) DO NOTHING;

-- Mark all published versions with published_at timestamp
UPDATE email_template_versions 
SET published_at = NOW() 
WHERE status = 'published' AND published_at IS NULL;
