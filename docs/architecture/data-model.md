# Data Model

Overview of the RustChat database schema.

## Entity Relationship Diagram

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   teams     │────<│ team_members│>────│   users     │
└─────────────┘     └─────────────┘     └─────────────┘
       │                                        │
       │         ┌─────────────┐               │
       └────────<│  channels   │>──────────────┘
                 └─────────────┘
                        │
                        │
                 ┌─────────────┐
                 │   posts     │
                 └─────────────┘
                        │
                 ┌─────────────┐
                 │   threads   │
                 └─────────────┘
```

## Core Tables

### users
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR(64) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    role VARCHAR(32) DEFAULT 'user',
    email_verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### teams
```sql
CREATE TABLE teams (
    id UUID PRIMARY KEY,
    name VARCHAR(64) NOT NULL,
    display_name VARCHAR(128),
    description TEXT,
    type VARCHAR(32) DEFAULT 'O', -- 'O' public, 'I' invite
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### channels
```sql
CREATE TABLE channels (
    id UUID PRIMARY KEY,
    team_id UUID REFERENCES teams(id),
    name VARCHAR(64) NOT NULL,
    display_name VARCHAR(128),
    type VARCHAR(32) DEFAULT 'O', -- 'O' public, 'P' private, 'D' direct
    creator_id UUID REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### posts
```sql
CREATE TABLE posts (
    id UUID PRIMARY KEY,
    channel_id UUID REFERENCES channels(id),
    user_id UUID REFERENCES users(id),
    message TEXT NOT NULL,
    type VARCHAR(32) DEFAULT '',
    parent_id UUID REFERENCES posts(id), -- For threads
    edit_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### files
```sql
CREATE TABLE files (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    size BIGINT NOT NULL,
    mime_type VARCHAR(255),
    path VARCHAR(512) NOT NULL, -- S3 key
    user_id UUID REFERENCES users(id),
    post_id UUID REFERENCES posts(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### sessions
```sql
CREATE TABLE sessions (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

### preferences
```sql
CREATE TABLE preferences (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id),
    category VARCHAR(64) NOT NULL,
    name VARCHAR(64) NOT NULL,
    value TEXT,
    UNIQUE(user_id, category, name)
);
```

## Migrations

Migrations are in `backend/migrations/`:
- Numbered sequentially
- Applied automatically at startup
- Irreversible (add columns, don't remove)

## Indexing Strategy

Key indexes for performance:
```sql
-- User lookups
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);

-- Channel membership
CREATE INDEX idx_channel_members_channel ON channel_members(channel_id);
CREATE INDEX idx_channel_members_user ON channel_members(user_id);

-- Posts (time-series)
CREATE INDEX idx_posts_channel_created ON posts(channel_id, created_at DESC);
CREATE INDEX idx_posts_user ON posts(user_id);

-- Sessions (cleanup)
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
```

---

*For implementation details: See backend models in `backend/src/models/`*
