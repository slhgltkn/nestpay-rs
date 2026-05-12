<div align="center">

# 🏦 nestpay-rs

**NestPay / Payten Sanal POS için Rust kütüphanesi**

Hash Version 3 (ver3) protokolünü destekler · Halkbank · Akbank · İşbank ve diğer NestPay entegrasyonları

[![Crates.io](https://img.shields.io/crates/v/nestpay?style=flat-square&color=fc8d62&logo=rust)](https://crates.io/crates/nestpay)
[![docs.rs](https://img.shields.io/docsrs/nestpay?style=flat-square&logo=docs.rs)](https://docs.rs/nestpay)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Build](https://img.shields.io/badge/tests-36%20passed-brightgreen?style=flat-square&logo=github-actions)](#-testler)

</div>

---

## 📋 İçindekiler

- [Özellikler](#-özellikler)
- [Desteklenen Modeller](#-desteklenen-modeller)
- [Kurulum](#-kurulum)
- [Hızlı Başlangıç](#-hızlı-başlangıç)
- [Desteklenen Modeller Detayı](#-model-detayları)
  - [3D Modeli](#1-3d-modeli)
  - [3D\_PAY Modeli](#2-3d_pay-modeli)
  - [3D\_PAY\_HOSTING Modeli](#3-3d_pay_hosting-modeli)
- [Ham Hash Hesaplama](#-ham-hash-hesaplama)
- [Response Doğrulama](#-response-doğrulama)
- [Alan Referansı](#-alan-referansı)
- [Güvenlik](#-güvenlik)
- [Testler](#-testler)
- [Katkıda Bulunma](#-katkıda-bulunma)
- [Lisans](#-lisans)

---

## ✨ Özellikler

| Özellik | Durum |
|---|---|
| Hash Version 3 (ver3) protokolü | ✅ |
| 3D, 3D\_PAY, 3D\_PAY\_HOSTING modelleri | ✅ |
| PHP / C# / Java referans implementasyonlarıyla doğrulandı | ✅ |
| Response hash doğrulama (`verify_hash`) | ✅ |
| Type-safe enum'lar (Para birimi, İşlem tipi, Kart tipi) | ✅ |
| Sıfır `unsafe` kod | ✅ |
| `no_std` uyumlu olmayan harici bağımlılık yok | ✅ |

---

## 🏪 Desteklenen Modeller

```
3D              → Kart bilgisi merchant sayfasında toplanır, 3DS akışı merchant'ta yönetilir
3D_PAY          → Kart bilgisi merchant'ta toplanır, ödeme banka tarafından işlenir
3D_PAY_HOSTING  → Kart giriş sayfası bankada barındırılır (merchant kart görmez)
```

---

## 📦 Kurulum

`Cargo.toml` dosyasına ekleyin:

```toml
[dependencies]
nestpay = "0.1"
```

---

## 🚀 Hızlı Başlangıç

```rust
use nestpay::{ThreeDRequest, NestPayRequest, TransactionType, Currency, CardType};

fn main() {
    let request = ThreeDRequest {
        client_id:    std::env::var("NESTPAY_CLIENT_ID").unwrap(),
        amount:       "150.00".into(),   // Kuruş değil, noktalı TL
        ok_url:       "https://siteniz.com/odeme/basarili".into(),
        fail_url:     "https://siteniz.com/odeme/basarisiz".into(),
        callback_url: "https://siteniz.com/odeme/callback".into(),
        tran_type:    TransactionType::Auth,
        instalment:   String::new(),     // Tek çekim için boş bırakın
        currency:     Currency::Try,
        rnd:          uuid_or_timestamp(),
        lang:         "tr".into(),
        pan:          "4111111111111111".into(),
        cv2:          "000".into(),
        exp_year:     "26".into(),
        exp_month:    "12".into(),
        card_type:    CardType::Visa,
        bill_to_name:    Some("Ad Soyad".into()),
        bill_to_company: None,
    };

    let store_key = std::env::var("NESTPAY_STORE_KEY").unwrap();

    // Hash dahil tüm form parametrelerini al
    let form_params = request.form_params(&store_key);

    // form_params'ı NestPay 3D-gate URL'sine POST edin
    for (key, value) in &form_params {
        println!("{key} = {value}");
    }
}

fn uuid_or_timestamp() -> String {
    // Her işlem için benzersiz bir değer üretin
    // Örnek: uuid::Uuid::new_v4().to_string()
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}
```

---

## 🗂 Model Detayları

### 1. 3D Modeli

Müşteri kart bilgilerini **merchant'ın kendi sayfasında** girer. Hash sunucu tarafında hesaplanır, form NestPay'e POST edilir.

```rust
use nestpay::{ThreeDRequest, NestPayRequest, TransactionType, Currency, CardType};

let request = ThreeDRequest {
    client_id:    "400841576".into(),
    amount:       "250.00".into(),
    ok_url:       "https://siteniz.com/ok".into(),
    fail_url:     "https://siteniz.com/fail".into(),
    callback_url: "https://siteniz.com/callback".into(),
    tran_type:    TransactionType::Auth,
    instalment:   "3".into(),  // 3 taksit
    currency:     Currency::Try,
    rnd:          "benzersiz-islem-id-001".into(),
    lang:         "tr".into(),
    pan:          "4111111111111111".into(),
    cv2:          "123".into(),
    exp_year:     "27".into(),
    exp_month:    "06".into(),
    card_type:    CardType::Visa,
    bill_to_name:    Some("Ahmet Yılmaz".into()),
    bill_to_company: None,
};

let params = request.form_params("STORE_KEY_BURAYA");
// params → NestPay 3D-gate'e POST edin
```

---

### 2. 3D\_PAY Modeli

Kart bilgisi merchant'ta toplanır ancak **ödemeyi banka işler**. Storetype: `3D_PAY`

```rust
use nestpay::{ThreeDPayRequest, NestPayRequest, TransactionType, Currency, CardType};

let request = ThreeDPayRequest {
    client_id:    "400841576".into(),
    amount:       "89.90".into(),
    ok_url:       "https://siteniz.com/ok".into(),
    fail_url:     "https://siteniz.com/fail".into(),
    callback_url: "https://siteniz.com/callback".into(),
    tran_type:    TransactionType::Auth,
    instalment:   String::new(),
    currency:     Currency::Try,
    rnd:          "benzersiz-islem-id-002".into(),
    lang:         "tr".into(),
    pan:          "5100000000000008".into(),
    cv2:          "456".into(),
    exp_year:     "28".into(),
    exp_month:    "09".into(),
    card_type:    CardType::MasterCard,
    bill_to_name: None,
    bill_to_company: Some("Firma A.Ş.".into()),
};

let params = request.form_params("STORE_KEY_BURAYA");
```

---

### 3. 3D\_PAY\_HOSTING Modeli

**Kart giriş sayfası bankada barındırılır.** Merchant kart numarası görmez; yalnızca işlem bilgilerini gönderir. Storetype: `3D_PAY_HOSTING`

```rust
use nestpay::{ThreeDPayHostingRequest, NestPayRequest, TransactionType, Currency};

let request = ThreeDPayHostingRequest {
    client_id:    "400841576".into(),
    amount:       "499.99".into(),
    ok_url:       "https://siteniz.com/ok".into(),
    fail_url:     "https://siteniz.com/fail".into(),
    callback_url: "https://siteniz.com/callback".into(),
    tran_type:    TransactionType::Auth,
    instalment:   String::new(),
    currency:     Currency::Try,
    rnd:          "benzersiz-islem-id-003".into(),
    lang:         "tr".into(),
    bill_to_name:    Some("Ayşe Kaya".into()),
    bill_to_company: None,
    refresh_time:    Some("5".into()), // 5 saniye sonra otomatik yönlendir
};

let params = request.form_params("STORE_KEY_BURAYA");
```

---

## 🔧 Ham Hash Hesaplama

Kendi parametre yapınız varsa veya `compute_hash`'i doğrudan kullanmak istiyorsanız:

```rust
use nestpay::hash::compute_hash;

let params = vec![
    ("clientid",      "400841576"),
    ("amount",        "150.00"),
    ("currency",      "949"),
    ("rnd",           "benzersiz-deger"),
    ("storetype",     "3d"),
    ("hashAlgorithm", "ver3"),
    // ... diğer parametreler
];

let hash = compute_hash(&params, "STORE_KEY_BURAYA");
println!("Hash: {hash}");
```

> **Not:** `hash`, `encoding` ve `countdown` parametreleri otomatik olarak hash hesaplamasından hariç tutulur.

---

## ✅ Response Doğrulama

Banka callback/response'unu aldıktan sonra hash'i doğrulayın:

```rust
use nestpay::hash::verify_hash;

// Bankadan gelen POST parametrelerini bir Vec'e dönüştürün
let response_params: Vec<(String, String)> = /* request.form() veya benzeri */;

// Gelen HASH değerini ayıklayın
let received_hash = /* response_params'tan "HASH" alanını çekin */;

// Kalan parametrelerle hash'i doğrulayın
let is_valid = verify_hash(&response_params, "STORE_KEY_BURAYA", &received_hash);

if is_valid {
    println!("✅ Hash doğrulandı — işleme devam edilebilir");
} else {
    println!("❌ Geçersiz imza — güvenlik ihlali olabilir");
}
```

---

## 📖 Alan Referansı

### Para Birimleri (`Currency`)

| Enum | Kod | Para Birimi |
|------|-----|-------------|
| `Currency::Try` | `949` | Türk Lirası |
| `Currency::Usd` | `840` | ABD Doları |
| `Currency::Eur` | `978` | Euro |
| `Currency::Gbp` | `826` | İngiliz Sterlini |
| `Currency::Jpy` | `392` | Japon Yeni |

### İşlem Tipleri (`TransactionType`)

| Enum | NestPay Değeri | Açıklama |
|------|----------------|----------|
| `TransactionType::Auth` | `Auth` | Satış / Otorizasyon |
| `TransactionType::PreAuth` | `PreAuth` | Ön Otorizasyon |
| `TransactionType::PostAuth` | `PostAuth` | Ön Otorizasyon Kapama |
| `TransactionType::Credit` | `Credit` | İade |
| `TransactionType::Void` | `Void` | İptal |

### Kart Tipleri (`CardType`)

| Enum | NestPay Değeri | Açıklama |
|------|----------------|----------|
| `CardType::Visa` | `1` | Visa |
| `CardType::MasterCard` | `2` | MasterCard |

### Taksit (`instalment`)

| Değer | Anlam |
|-------|-------|
| `""` (boş string) | Tek çekim |
| `"2"` | 2 taksit |
| `"3"` | 3 taksit |
| `"6"`, `"9"`, `"12"` | … |

---

## 🔐 Güvenlik

> **StoreKey'inizi asla kaynak koduna yazmayın.**

Doğru kullanım — ortam değişkeni veya secret manager:

```rust
// ✅ Doğru
let store_key = std::env::var("NESTPAY_STORE_KEY")
    .expect("NESTPAY_STORE_KEY ortam değişkeni tanımlanmamış");

// ❌ Yanlış — kaynak koduna gömülü
let store_key = "KajWWzeBWL7h3qJ";
```

`.env` dosyası örneği (`.gitignore`'a eklemeyi unutmayın):

```env
NESTPAY_CLIENT_ID=400841598
NESTPAY_STORE_KEY=buraya_gercek_store_key
NESTPAY_API_USER=api
NESTPAY_API_PASS=buraya_gercek_sifre
```

`.gitignore`:

```
.env
.env.*
*.secret
```

---

## 🧪 Testler

```bash
# Tüm testleri çalıştır
cargo test

# Çıktılarla birlikte çalıştır
cargo test -- --nocapture

# Belirli bir testi çalıştır
cargo test doc_example -- --nocapture
```

**Test kapsamı:**

| Kategori | Test Sayısı | Kapsam |
|----------|-------------|--------|
| Hash algoritması (birim) | 6 | Escape, sıralama, hariç tutma |
| Entegrasyon testleri | 25 | Model alanları, round-trip, edge case'ler |
| Dokümantasyon testleri | 5 | Kod örnekleri |
| **Toplam** | **36** | |

Hash doğruluğu; PHP `natcasesort + hash('sha512') + base64_encode`, C# `SHA512CryptoServiceProvider` ve Java `MessageDigest` referans implementasyonlarıyla **çapraz doğrulanmıştır**.

---

## 🏃 Örnekler

```bash
# 3D modeli örneği
cargo run --example three_d_payment

# 3D_PAY modeli örneği
cargo run --example three_d_pay_payment

# 3D_PAY_HOSTING modeli örneği
cargo run --example three_d_pay_hosting_payment
```

---

## 🤝 Katkıda Bulunma

Katkılarınızı bekliyoruz! Özellikle:

- **Eksik bankalar** — Diğer NestPay entegrasyon bankalarına ait örnek/test
- **Yeni işlem tipleri** — `PostAuth`, `Credit`, `Void` için tam örnekler
- **Web framework entegrasyonu** — Axum, Actix-web, Rocket için örnek kodlar
- **Hata düzeltmeleri** — Tespit ettiğiniz uyumsuzluklar

### Geliştirme

```bash
git clone https://github.com/slhgltkn/nestpay-rs
cd nestpay-rs
cargo test          # Testler geçmeli
cargo clippy        # Uyarı olmamalı
cargo fmt --check   # Format uyumlu olmalı
```

---

## 📜 Algoritma — Hash Version 3

> Resmi NestPay Hash Ver3 Dokümantasyonu (2024) baz alınmıştır.

```
1. Tüm parametreler alfabetik sırayla (A→Z) case-insensitive sıralanır
2. "hash", "encoding", "countdown" parametreleri hariç tutulur
3. Her değerdeki  \  karakteri  \\  olarak,  |  karakteri  \|  olarak escaped edilir
4. Değerler | karakteriyle birleştirilir (her değerin sonuna | eklenir)
5. Sona | + escaped storeKey eklenir (trailing | olmadan)
6. Elde edilen plaintext SHA-512 ile özetlenir (UTF-8 encoding)
7. SHA-512 çıktısı Base64 (standart alfabe) ile encode edilir
```

**Örnek plaintext yapısı:**

```
95.93|billToCompany|name|https://.../callback|100200127|949|https://.../fail|ver3||tr|https://.../ok|rnd_degeri|3d|Auth|STORE_KEY
```

---

## 📄 Lisans

[MIT](LICENSE) © 2026 [Salih Gültekin](https://github.com/slhgltkn)
