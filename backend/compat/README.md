# Mattermost Mobile Compatibility Tests

This directory contains contract tests and trace-based replay tools for verifying 
RustChat's compatibility with the Mattermost Mobile client.

## Structure

```
compat/
├── traces/          # mitmproxy captures from real mobile sessions
├── contracts/       # JSON schemas for expected response shapes
└── tests/           # Integration test implementations
```

## Running Tests

### Contract Tests (Rust)
```bash
cargo test --lib compat::
```

### Trace Replay
```bash
python3 replay_traces.py \
  --trace traces/mobile_session.mitm \
  --target http://localhost:8080 \
  --output diff_report.json
```

## Capturing Traces

```bash
# Start mitmproxy as reverse proxy
mitmproxy --mode reverse:http://rustchat:8080 \
          -w traces/session.mitm \
          --set flow_detail=3

# Point mobile to mitmproxy address
```

## Schema Validation

JSON schemas in `contracts/` define expected response shapes.
See `contracts/user.schema.json` for examples.
