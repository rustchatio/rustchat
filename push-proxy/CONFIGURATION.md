# Configuration Guide: RustChat Push Proxy

This guide explains how to configure the Push Proxy service with your Firebase credentials.

## 1. Firebase Service Account Key

To interact with the Firebase HTTP v1 API, you need a Google Service Account JSON key.

1. Go to the [Firebase Console](https://console.firebase.google.com/).
2. Select your project.
3. Go to **Project Settings** > **Service accounts**.
4. Click **Generate new private key** and download the JSON file.
5. Save this file as `firebase-key.json` in your project's `secrets/` directory (or any secure location).

## 2. Environment Variables

The service requires the following environment variables:

| Variable | Description | Example |
| :--- | :--- | :--- |
| `FIREBASE_PROJECT_ID` | Your Firebase Project ID | `rustchat-prod-123` |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to the service account JSON file | `/etc/firebase/service-account.json` |
| `RUST_LOG` | Logging level | `push_proxy=info,tower_http=info` |

## 3. Docker Sidecar Configuration

The recommended way to run the service is as a Docker sidecar alongside the main `rustchat-server`.

```yaml
services:
  push-proxy:
    image: rustchat/push-proxy:latest
    environment:
      - FIREBASE_PROJECT_ID=your-project-id
      - GOOGLE_APPLICATION_CREDENTIALS=/etc/firebase/service-account.json
    volumes:
      # Mount the secret key as read-only
      - ./secrets/firebase-key.json:/etc/firebase/service-account.json:ro
    networks:
      - rustchat-internal
    restart: unless-stopped
```

## 4. Security Notes

- **Read-Only Mounts:** Always mount the service account key as `:ro` (read-only) in Docker.
- **Internal Network:** Ensure the `push-proxy` only listens on an internal Docker network. It should **not** expose port `3000` to the public internet.
- **Secrets Management:** Do not commit the `firebase-key.json` file to version control. Use a `.gitignore` to exclude your `secrets/` folder.
