# RustChat Documentation

Welcome to the RustChat documentation. This directory contains all project documentation organized by audience and purpose.

## Quick Navigation

| If you're a... | Start here |
|----------------|------------|
| **End User** looking to use RustChat | [User Guide](./user/) |
| **Administrator** deploying or managing RustChat | [Admin Guide](./admin/) |
| **Developer** contributing code | [Development Guide](./development/) |
| **Operator** handling incidents | [Operations](./operations/) |

---

## Documentation Structure

### 📖 [User Documentation](./user/)
End-user guides for using RustChat features.
- Quick start and onboarding
- Feature guides (channels, messages, calls)
- Troubleshooting common issues

### 🔧 [Admin Documentation](./admin/)
Guides for system administrators and operators.
- Installation and deployment
- Configuration reference
- Security hardening
- Scaling guidelines
- SSO and email setup

### 🏗️ [Architecture](./architecture/)
System architecture and design documentation.
- High-level system overview
- Backend architecture
- Frontend architecture
- WebSocket and real-time systems
- Calls deployment modes

### 👨‍💻 [Development](./development/)
Resources for contributors and developers.
- Contributing guidelines
- Development setup
- Testing practices
- Code style and conventions
- Mattermost compatibility requirements
- Agent operating model

### 🚨 [Operations](./operations/)
Operational runbooks and procedures.
- Incident response
- Debugging guides
- Maintenance procedures

### 📚 [Reference](./reference/)
Quick reference materials.
- API documentation
- Environment variables
- Mobile compatibility matrix

### 📋 [Decisions](./decisions/)
Architecture Decision Records (ADRs).
- Historical technical decisions
- Design rationale

### 📦 [Archive](./archive/)
Completed work and historical documentation.
- Phase completion reports
- Implementation summaries
- Superseded plans

---

## Key Documents by Topic

### Getting Started
- [Quick Start (Users)](./user/quick-start.md)
- [Installation Guide (Admins)](./admin/installation.md)
- [Local Development Setup](./development/local-setup.md)

### Security
- [Security Deployment Guide](./admin/security.md)
- [SSO Configuration](./admin/sso.md)
- [Security Policy](../SECURITY.md)

### Compatibility
- [Mattermost Compatibility](./development/compatibility.md)
- [Mobile Compatibility Matrix](./reference/compatibility-matrix.md)

### Architecture
- [System Overview](./architecture/overview.md)
- [Backend Deep Dive](./architecture/backend.md)
- [WebSocket Architecture](./architecture/websocket.md)

---

## Contributing to Documentation

When adding new documentation:
1. Place it in the appropriate section based on audience
2. Update this README with a link
3. Follow the existing style and formatting
4. For significant changes, consider creating an ADR in `./decisions/`

---

*Last updated: 2026-04-05*
