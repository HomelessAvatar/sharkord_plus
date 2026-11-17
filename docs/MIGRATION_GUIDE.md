# Migrating from Vanilla Sharkord to Sharkord Plus (VPS Migration Guide)

This guide provides step-by-step instructions for upgrading an existing standard (vanilla) **Sharkord** VPS installation to **Sharkord Plus** with **zero data loss** and minimal downtime.

---

## 🎯 1. Why Upgrade to Sharkord Plus?

Sharkord Plus is an enhanced, production-ready distribution built upon the upstream Sharkord architecture, introducing key enhancements:

* **100% Backward Data Compatibility:** Your existing SQLite database (`db.sqlite`), user accounts, password hashes, server permissions, channel structure, chat history, and `uploads/` directory remain fully intact without destructive migrations.
* **Live Database & Webhook Synchronization:** When an external community portal, bot, or script modifies user records in SQLite, Sharkord Plus detects the change via `PRAGMA data_version` and immediately broadcasts realtime `USER_UPDATE` events to all connected web and desktop clients. A dedicated `trpc.users.sync` endpoint is also available for instant webhook triggers.
* **Audio & Screen Share Priority (WebRTC Pacer):** Prevents video packets from starving audio bandwidth during high-bitrate screen sharing (`high` audio priority / `low` video priority), resolving choppy or frozen voice communication.
* **Full-Screen Window Overlays:** Resolves UI clipping and frozen controls during full-screen video and screen share sessions.
* **Pure Native GPUI Desktop Client:** Seamlessly integrates with our ultra-lightweight (35 MB RAM, 120 FPS GPU-accelerated) native Rust desktop application (`apps/desktop-gpui`), completely free of Electron overhead.
* **Automated Container Updates:** Official multi-arch Docker images on GitHub Container Registry (GHCR) with native What's Up Docker (WUD) labels and Semantic Versioning (SemVer) tags.

---

## 🛡️ 2. Step 1: Create a Full Backup

Before performing any upgrade, create a complete backup of your persistent data directory (typically `./data` or `/home/bun/.config/sharkord` or `~/.config/sharkord`):

```bash
# 1. Navigate to your existing Sharkord directory
cd /opt/sharkord   # Replace with your actual installation directory

# 2. Stop the running service
docker compose down
# Or if running via systemd:
# sudo systemctl stop sharkord

# 3. Create a timestamped compressed archive of the data directory
tar -czvf sharkord_backup_$(date +%Y%m%d_%H%M%S).tar.gz ./data

# Alternatively, create a direct directory copy:
cp -r ./data ./data_backup_$(date +%Y%m%d)
```

> [!TIP]
> If you wish to capture an online SQLite backup while the server is active, you can run:
> ```bash
> sqlite3 ./data/db.sqlite ".backup ./data/db_backup.sqlite"
> ```

---

## 🚀 3. Step 2: Choose Your Deployment Method

Select the method matching your VPS environment:

---

### Method A: Docker / Docker Compose (Recommended)

If your existing server runs via Docker Compose, upgrading takes less than two minutes.

#### 1. Clone the Sharkord Plus Repository or Update Compose
```bash
# Navigate to your deployment root
cd /opt

# Clone the Sharkord Plus repository
git clone https://github.com/HomelessAvatar/sharkord_plus.git sharkord-plus
cd sharkord-plus

# Copy your existing vanilla Sharkord data directory
cp -r /opt/sharkord/data ./data
```

#### 2. Review `docker-compose.yml`
The repository includes a production-ready `docker-compose.yml`:

```yaml
services:
  sharkord-plus:
    # SemVer release image (fully compatible with What's Up Docker / WUD automated updates)
    image: ghcr.io/homelessavatar/sharkord_plus:0.2.0
    container_name: sharkord-plus
    restart: unless-stopped
    labels:
      # What's Up Docker (WUD) watch & update labels
      - "wud.watch=true"
      - "wud.tag.include=^\\d+\\.\\d+\\.\\d+.*$"
      - "wud.link.template=https://github.com/HomelessAvatar/sharkord_plus/releases/tag/v$${major}.$${minor}.$${patch}"
    ports:
      # HTTP / WebSocket (tRPC API & Web Client)
      - "4991:4991/tcp"
      # Mediasoup Voice & Screen Share WebRTC SFU port range
      - "40000-40050:40000-40050/udp"
      - "40000-40050:40000-40050/tcp"
    environment:
      - PUID=1000
      - PGID=1000
      # VPS external public IP or DDNS hostname (Required for WebRTC voice & video ICE candidates)
      # - SHARKORD_WEBRTC_ANNOUNCED_ADDRESS=203.0.113.195
    volumes:
      # Persistent Sharkord data directory (db.sqlite, uploads, logs)
      - ./data:/home/bun/.config/sharkord
```

#### 3. Pull Image and Start the Container
```bash
# Pull the pre-built multi-arch image from GHCR
docker compose pull

# Start in detached mode
docker compose up -d
```

#### 4. Verify Server Logs
```bash
docker compose logs -f
```
You should see database migrations check passing without errors and the server listening on port `4991`.

---

### Method B: Bare-Metal / Systemd Service

If your VPS runs Sharkord directly with Bun as a systemd service:

#### 1. Stop the Existing Service
```bash
sudo systemctl stop sharkord
```

#### 2. Clone the Plus Repository and Install Dependencies
```bash
cd /opt
git clone https://github.com/HomelessAvatar/sharkord_plus.git sharkord-plus
cd sharkord-plus

# Install workspace dependencies with Bun
bun install
```

#### 3. Build Web Client and Server
```bash
cd apps/server
bun run build
```
The compiled single-file binary will be generated at `apps/server/build/out/sharkord-linux-x64`.

#### 4. Update the Systemd Service Unit
Edit `/etc/systemd/system/sharkord.service`:

```ini
[Unit]
Description=Sharkord Plus Server
After=network.target

[Service]
Type=simple
User=bun
WorkingDirectory=/opt/sharkord-plus
# Path to your existing persistent data folder
Environment=SHARKORD_DATA_PATH=/opt/sharkord-plus/data
Environment=SHARKORD_ENV=production
Environment=PORT=4991
# Optional: External public IP for WebRTC SFU
# Environment=SHARKORD_WEBRTC_ANNOUNCED_ADDRESS=203.0.113.195

# Run using the compiled standalone binary:
ExecStart=/opt/sharkord-plus/apps/server/build/out/sharkord-linux-x64

# Or run directly with Bun:
# ExecStart=/usr/local/bin/bun /opt/sharkord-plus/apps/server/src/index.ts

Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

#### 5. Reload and Restart
```bash
sudo systemctl daemon-reload
sudo systemctl start sharkord
sudo systemctl status sharkord
journalctl -u sharkord -f
```

---

### Method C: Standalone Linux Executable Binary

To run without Docker or compiling locally:

```bash
# 1. Stop existing service
sudo systemctl stop sharkord

# 2. Download the latest compiled release binary
cd /opt/sharkord
wget -O sharkord-plus https://github.com/HomelessAvatar/sharkord_plus/releases/latest/download/sharkord-linux-x64
chmod +x sharkord-plus

# 3. Set file permissions
chown bun:bun sharkord-plus

# 4. Point systemd ExecStart to the new binary and restart
sudo systemctl restart sharkord
```

---

## 🌐 4. Reverse Proxy & Network Configuration

Sharkord Plus utilizes HTTP, tRPC, WebSocket subscriptions, and WebRTC SFU media streams.

### Nginx Configuration (`/etc/nginx/sites-available/sharkord.conf`)

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

server {
    server_name sharkord.yourdomain.com;

    # Maximum file upload size
    client_max_body_size 100M;

    location / {
        proxy_pass http://127.0.0.1:4991;
        proxy_http_version 1.1;

        # WebSocket & tRPC realtime subscriptions headers
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Keepalive timeouts for persistent WebSocket connections
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }

    listen 443 ssl http2;
    ssl_certificate /etc/letsencrypt/live/sharkord.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/sharkord.yourdomain.com/privkey.pem;
}
```

### Firewall (UFW) Rules
Ensure Mediasoup WebRTC media ports are open to the internet:

```bash
# Standard web ports
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Direct HTTP port (if not using reverse proxy)
sudo ufw allow 4991/tcp

# Mediasoup Voice & Screen Share WebRTC SFU port range (CRITICAL)
sudo ufw allow 40000:40050/udp
sudo ufw allow 40000:40050/tcp

sudo ufw reload
```

---

## ✅ 5. Verification & Testing

Verify that all subsystems operate as expected after migration:

### 1. Healthcheck Endpoint
```bash
curl -I http://localhost:4991/healthz
# Should return: HTTP/1.1 200 OK
```

### 2. Live Profile Sync Test
To test the real-time user sync mechanism:
```bash
# Send an authenticated user sync trigger
curl -X POST http://localhost:4991/trpc/users.sync \
  -H "Content-Type: application/json" \
  -d '{}'
```
Connected web and desktop clients should update instantly without requiring a page refresh.

### 3. Voice & Screen Sharing Test
* Connect to a voice channel via Web or Native GPUI Desktop client.
* Start a screen share session at 1080p 60 FPS; confirm audio transmission remains fluid with zero packet drop due to the WebRTC pacer priority engine.

---

## 🔄 6. Rollback Plan

If you need to revert to your previous vanilla Sharkord installation for any reason:

1. **Stop the Plus service:**
   ```bash
   docker compose down
   # Or: sudo systemctl stop sharkord
   ```

2. **Restore your database and uploads backup:**
   ```bash
   rm -rf ./data
   cp -r ./data_backup_YYYYMMDD ./data
   ```

3. **Restart the original Sharkord container or service:**
   ```bash
   cd /opt/sharkord
   docker compose up -d
   # Or: sudo systemctl start sharkord
   ```

Your server will resume running on standard vanilla Sharkord with complete data continuity.

---

## 💬 Support & Contribution
For inquiries, bug reports, or feature requests, visit [GitHub Issues](https://github.com/HomelessAvatar/sharkord_plus/issues).
