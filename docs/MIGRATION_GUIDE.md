# Standart Sharkord'dan Sharkord Plus'a Geçiş Rehberi (VPS Migration Guide)

Bu doküman, mevcut bir VPS sunucusunda çalışan standart (vanilla) **Sharkord** sunucunuzu, hiçbir veri kaybı yaşamadan ve kesintisiz şekilde **Sharkord Plus** sürümüne nasıl yükselteceğinizi adım adım açıklar.

---

## 🎯 1. Neden Sharkord Plus'a Geçmelisiniz?

Sharkord Plus sunucusu, standart Sharkord mimarisini bozmadan üzerine aşağıdaki kritik iyileştirmeleri ekler:

* **100% Geriye Dönük Veri Uyumluluğu:** Mevcut `db.sqlite` veritabanınız, kullanıcı hesapları, şifreler, roller, sunucu ayarları, mesaj geçmişi ve `uploads/` dizinindeki tüm dosyalar aynen korunur.
* **Canlı Veritabanı Senkronizasyonu (`PRAGMA data_version`):** Dışarıdan bir topluluk web sitesi, bot veya script SQLite veritabanındaki kullanıcı bilgilerini güncellediğinde, sunucu bunu anında algılar ve tüm bağlı istemcilere (Web ve Masaüstü) anlık `USER_UPDATE` olayı yayınlar.
* **Anlık Webhook Senkronizasyonu:** `trpc.users.sync` endpoint'i ile admin panelleri veya harici sistemler tek çağrıyla kullanıcı profil değişikliklerini anında istemcilere yansıtabilir.
* **Gelişmiş Ses & Ekran Paylaşımı Önceliği (WebRTC Pacer):** Ekran paylaşımı açıldığında video paketlerinin ses paketlerini ezmesi ve seste takılma/kesilme yaşanması (`high` audio priority / `low` video priority) engellenmiştir.
* **Tam Ekran Menü ve Panel Desteği:** Tam ekran ekran paylaşımı sırasında ses kontrolleri, kullanıcı panelleri ve menülerin donması / görünmez olması sorunu çözülmüştür.
* **Pure Native GPUI İstemci Desteği:** 120 FPS GPU hızlandırmalı, ultra hafif (35 MB RAM) yerel Rust masaüstü istemcimizle tam uyumludur.

---

## 🛡️ 2. Ön Hazırlık: Veri Yedekleme (Backup)

Geçiş işlemine başlamadan önce mevcut veri klasörünüzü mutlaka yedekleyin.

Mevcut Sharkord kurulumunuzun veri dizinine gidin (genellikle `./data` veya `/home/bun/.config/sharkord` ya da `~/.config/sharkord`):

```bash
# 1. Sharkord çalışma dizinine gidin
cd /opt/sharkord   # kendi kurulum dizininizi yazın

# 2. Servisi geçici olarak durdurun
docker compose down
# veya systemd kullanıyorsanız:
# sudo systemctl stop sharkord

# 3. Veri dizininin zaman damgalı tam bir yedeğini alın
tar -czvf sharkord_backup_$(date +%Y%m%d_%H%M%S).tar.gz ./data

# Alternatif olarak klasörü kopyalayın:
cp -r ./data ./data_backup_$(date +%Y%m%d)
```

> [!TIP]
> SQLite çalışırken canlı yedek almak isterseniz `sqlite3 ./data/db.sqlite ".backup ./data/db_backup.sqlite"` komutunu da çalıştırabilirsiniz.

---

## 🚀 3. Geçiş Yöntemleri

Kullandığınız altyapıya göre aşağıdaki 3 yöntemden birini seçin:

---

### Yöntem A: Docker / Docker Compose (Önerilen En Yaygın Yöntem)

Mevcut Sharkord sunucunuz Docker Compose ile çalışıyorsa geçiş işlemi yalnızca birkaç dakikanızı alır.

#### 1. Proje Deposunu Güncelleyin veya Klonlayın
```bash
# Mevcut dizinden Sharkord Plus reposuna geçin:
cd /opt

# Sharkord Plus'ı klonlayın (veya mevcut repoda remote'u güncelleyin)
git clone https://github.com/HomelessAvatar/sharkord_plus.git sharkord-plus
cd sharkord-plus

# Eski veri klasörünüzü yeni konuma taşıyın/kopyalayın
cp -r /opt/sharkord/data ./data
```

#### 2. `docker-compose.yml` Dosyasını Kontrol Edin
Depo kökünde yer alan `docker-compose.yml` dosyası hazır olarak gelmektedir:

```yaml
services:
  sharkord-plus:
    build:
      context: .
      dockerfile: Dockerfile
    image: homelessavatar/sharkord_plus:latest
    container_name: sharkord-plus
    restart: unless-stopped
    ports:
      - "4991:4991/tcp"
      - "40000-40050:40000-40050/udp"
      - "40000-40050:40000-40050/tcp"
    environment:
      - PUID=1000
      - PGID=1000
      # VPS dış IP adresiniz (WebRTC ses/video bağlantıları için gereklidir)
      # - SHARKORD_WEBRTC_ANNOUNCED_ADDRESS=203.0.113.195
    volumes:
      - ./data:/home/bun/.config/sharkord
```

#### 3. İmajı Derleyin ve Başlatın
```bash
docker compose up -d --build
```

#### 4. Logları Kontrol Edin
```bash
docker compose logs -f
```
Loglarda veritabanı migrasyonlarının tamamlandığını ve sunucunun `4991` portunda dinlemeye başladığını görmelisiniz.

---

### Yöntem B: Bare-Metal / Systemd ile Doğrudan VPS Üzerinde Çalıştırma

Eğer sunucunuzda doğrudan Bun kuruluysa ve Sharkord bir `systemd` servisi olarak çalışıyorsa:

#### 1. Servisi Durdurun
```bash
sudo systemctl stop sharkord
```

#### 2. Plus Sürümünü Klonlayın ve Bağımlılıkları Kurun
```bash
cd /opt
git clone https://github.com/HomelessAvatar/sharkord_plus.git sharkord-plus
cd sharkord-plus

# Bun ile bağımlılıkları yükleyin
bun install
```

#### 3. Web İstemcisini ve Sunucuyu Derleyin (Production Build)
```bash
cd apps/server
bun run build
```
Derleme sonucu `apps/server/build/out/sharkord-linux-x64` ikili dosyası üretilir.

#### 4. Systemd Servis Dosyasını Güncelleyin
`/etc/systemd/system/sharkord.service` dosyanızı düzenleyin:

```ini
[Unit]
Description=Sharkord Plus Server
After=network.target

[Service]
Type=simple
User=bun
WorkingDirectory=/opt/sharkord-plus
# Veri dizini ortam değişkeni (eski data dizininizin yolu)
Environment=SHARKORD_DATA_PATH=/opt/sharkord-plus/data
Environment=SHARKORD_ENV=production
Environment=PORT=4991
# İsteğe bağlı: WebRTC dış IP adresiniz
# Environment=SHARKORD_WEBRTC_ANNOUNCED_ADDRESS=203.0.113.195

# Derlenmiş tek binary ile çalıştırma:
ExecStart=/opt/sharkord-plus/apps/server/build/out/sharkord-linux-x64

# Veya Bun ile doğrudan kaynak koddan çalıştırma:
# ExecStart=/usr/local/bin/bun /opt/sharkord-plus/apps/server/src/index.ts

Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

#### 5. Servisi Başlatın ve Durumu Kontrol Edin
```bash
sudo systemctl daemon-reload
sudo systemctl start sharkord
sudo systemctl status sharkord
journalctl -u sharkord -f
```

---

### Yöntem C: Bağımsız İkili Dosya (Standalone Linux Binary) ile Geçiş

Herhangi bir derleme aracı veya Docker kurmadan, hazır Linux binary'sini kullanmak için:

```bash
# 1. Eski servisi durdurun
sudo systemctl stop sharkord

# 2. GitHub Releases üzerinden en güncel sharkord-linux-x64 dosyasını indirin
cd /opt/sharkord
wget -O sharkord-plus https://github.com/HomelessAvatar/sharkord_plus/releases/latest/download/sharkord-linux-x64
chmod +x sharkord-plus

# 3. İzinleri ayarlayın
chown bun:bun sharkord-plus

# 4. Servisin ExecStart yolunu yeni dosyaya yönlendirip başlatın
sudo systemctl restart sharkord
```

---

## 🌐 4. Tersine Proxy (Reverse Proxy) & Ağ Yapılandırması

Sharkord Plus; HTTP, tRPC ve WebRTC protokollerini kullanır.

### Nginx Yapılandırması (Örnek `/etc/nginx/sites-available/sharkord.conf`)

```nginx
map $http_upgrade $connection_upgrade {
    default upgrade;
    ''      close;
}

server {
    server_name sharkord.alanadiniz.com;

    # Dosya yüklemeleri için boyut sınırı
    client_max_body_size 100M;

    location / {
        proxy_pass http://127.0.0.1:4991;
        proxy_http_version 1.1;

        # WebSocket & tRPC realtime subscriptions için zorunlu başlıklar
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Zaman aşımı ayarları (uzun süreli WebSocket bağlantıları için)
        proxy_read_timeout 86400s;
        proxy_send_timeout 86400s;
    }

    listen 443 ssl http2;
    # SSL sertifika yollarınız (Certbot vb.)
    ssl_certificate /etc/letsencrypt/live/sharkord.alanadiniz.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/sharkord.alanadiniz.com/privkey.pem;
}
```

### Güvenlik Duvarı (UFW) Ayarları
Ses ve ekran paylaşımı paketlerinin sunucuya doğrudan UDP üzerinden ulaşabilmesi gerekir:

```bash
# Web ve Reverse Proxy portları
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Eğer doğrudan porttan bağlanıyorsanız
sudo ufw allow 4991/tcp

# Mediasoup WebRTC Ses ve Video Port Aralığı (ÖNEMLİ!)
sudo ufw allow 40000:40050/udp
sudo ufw allow 40000:40050/tcp

sudo ufw reload
```

---

## ✅ 5. Geçiş Sonrası Doğrulama ve Testler

Geçiş tamamlandıktan sonra aşağıdaki kontrolleri yaparak sistemin kusursuz çalıştığından emin olun:

### 1. Servis Sağlık Kontrolü
```bash
curl -I http://localhost:4991/healthz
# HTTP/1.1 200 OK dönmelidir
```

### 2. Canlı Kullanıcı Senkronizasyonunun Test Edilmesi
Sharkord Plus'ın yeni `trpc.users.sync` özelliğini test etmek için:
```bash
# Admin yetkisine sahip bir oturum ile kullanıcı senkronizasyonunu tetikleyin:
curl -X POST http://localhost:4991/trpc/users.sync \
  -H "Content-Type: application/json" \
  -d '{}'
```
Tüm bağlı masaüstü ve web kullanıcılarında profillerin anında yenilendiğini gözlemleyebilirsiniz.

### 3. Ses ve Ekran Paylaşımı
* Web veya GPUI Desktop istemcisi ile ses kanalına katılın.
* Ekran paylaşımı açın ve ses paketlerinin kesilmeden, pacer önceliğiyle aktığını teyit edin.

---

## 🔄 6. Sorun Giderme ve Geri Alma (Rollback Planı)

Eğer herhangi bir sebeple orijinal Sharkord sürümüne geri dönmek isterseniz:

1. **Plus servisini durdurun:**
   ```bash
   docker compose down
   # veya: sudo systemctl stop sharkord
   ```

2. **Yedeğinizi geri yükleyin:**
   ```bash
   rm -rf ./data
   cp -r ./data_backup_YYYYMMDD ./data
   ```

3. **Eski Sharkord imajını veya servisini yeniden başlatın:**
   ```bash
   cd /opt/sharkord
   docker compose up -d
   # veya: sudo systemctl start sharkord
   ```

Tüm verileriniz ve ayarlarınız önceki haline eksiksiz dönecektir.

---

## 💬 Destek ve Katkı
Geçiş sırasında herhangi bir soru veya sorunla karşılaşırsanız [GitHub Issues](https://github.com/HomelessAvatar/sharkord_plus/issues) üzerinden bize ulaşabilirsiniz.
