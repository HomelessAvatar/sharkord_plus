# Ana Depodaki (Upstream) Güncellemeleri Selektif Olarak Dahil Etme Rehberi

Evet, kesinlikle dahil edebilirsiniz! Projeyi forkladıktan sonra git remote yapımız otomatik olarak şu şekilde yapılandırıldı:

- **origin** -> `https://github.com/HomelessAvatar/sharkord_plus.git` (Sizin reponuz)
- **upstream** -> `https://github.com/Sharkord/sharkord.git` (Orijinal Sharkord reposu)

---

## Nasıl Güncelleme Alınır?

Ana projeye (`upstream`) yeni özellikler veya hata düzeltmeleri (bug fix) geldiğinde bunları projenize selektif olarak almak için şu adımları izleyebilirsiniz:

### 1. Upstream Değişikliklerini Çekin
```bash
git fetch upstream
```

### 2. Gelen Yeni Değişiklikleri İnceleyin
```bash
# Son commit'leri listeleyin
git log upstream/development --oneline -n 20

# Belirli bir commit'in içeriğini görmek için
git show <commit-hash>
```

### 3. Tek Bir Düzeltmeyi / Commit'i Projenize Dahil Edin (Cherry-pick)
Ana projedeki sadece belirli bir bug fix veya özelliği almak istiyorsanız:
```bash
git cherry-pick <commit-hash>
```
*Eğer çakışma (conflict) çıkarsa çözüp `git cherry-pick --continue` demeniz yeterlidir.*

### 4. Belli Bir Dalı veya Özelliği Birleştirmek (Merge)
Ana projedeki bir dalı doğrudan projenize katmak isterseniz:
```bash
git merge upstream/development
```

Böylece **Sharkord Plus** tamamen bağımsız kalırken, ana depodaki dilediğiniz geliştirmeyi projenizi bozmadan tek tek seçip alabilirsiniz.
