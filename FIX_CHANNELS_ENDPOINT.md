# Fix: Added Missing /channels Endpoint for Calls Plugin

## Problem
The frontend was calling `GET /api/v4/plugins/com.mattermost.calls/channels?mobilev2=true` and getting a 501 Not Implemented error.

## Solution
Added the missing endpoint to return channels with active calls.

## Changes Made

### 1. backend/src/api/v4/calls_plugin/mod.rs
- Added route: `GET /plugins/com.mattermost.calls/channels`
- Added handler `get_channels` that returns channels with active calls
- Added `CallChannelInfo` response struct

### 2. backend/src/api/v4/calls_plugin/state.rs
- Added `get_participant_count()` method to `CallStateManager`

## Deploy

```bash
docker-compose build backend
docker-compose up -d backend
```

## Response Format

The endpoint returns a list of channels with active calls that the user is a member of:

```json
[
  {
    "channel_id": "base64encodedid",
    "call_id": "base64encodedid",
    "enabled": true,
    "has_call": true,
    "participant_count": 3
  }
]
```
