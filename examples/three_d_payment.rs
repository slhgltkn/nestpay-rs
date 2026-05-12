//! Örnek: **3D** modeli ile ödeme isteği oluşturma
//!
//! Çalıştırmak için:
//! ```
//! cargo run --example three_d_payment
//! ```
//!
//! Bu örnek, PHP'deki `ver3RequestExample.php` dosyasının Rust karşılığıdır.

use nestpay::{CardType, Currency, NestPayRequest, ThreeDRequest, TransactionType};

fn main() {
    // Gerçek entegrasyonda `rnd` her işlem için benzersiz olmalıdır.
    // Örnek: uuid::Uuid::new_v4().to_string() veya SystemTime::now() tabanlı bir değer.
    let rnd = "0.56435200 1715000000"; // microtime() benzeri sabit değer (örnek için)

    let request = ThreeDRequest {
        client_id: "100200127".into(),
        amount: "91.96".into(),
        ok_url: "https://yoursite.com/ok".into(),
        fail_url: "https://yoursite.com/fail".into(),
        callback_url: "https://yoursite.com/callback".into(),
        tran_type: TransactionType::Auth,
        instalment: String::new(), // Tek çekim
        currency: Currency::Try,
        rnd: rnd.into(),
        lang: "tr".into(),
        pan: "4111111111111111".into(),
        cv2: "000".into(),
        exp_year: "26".into(),
        exp_month: "12".into(),
        card_type: CardType::Visa,
        bill_to_name: Some("Ad Soyad".into()),
        bill_to_company: Some("Şirket A.Ş.".into()),
    };

    let store_key = "TEST1234";

    // Sadece hash değerini hesapla
    let hash = request.compute_hash(store_key);
    println!("Hesaplanan Hash: {hash}");

    // 3D-gate'e POST edilecek tüm parametreler (hash dahil)
    let form_params = request.form_params(store_key);

    println!("\n--- NestPay 3D Gate'e gönderilecek form parametreleri ---");
    for (key, value) in &form_params {
        println!("{key:<40} = {value}");
    }

    println!("\n--- Aynı parametrelerle HTML form örneği ---");
    println!(
        r#"<form method="post" action="https://<3dgate_host>/<3dgate_path>">"#
    );
    for (key, value) in &form_params {
        // Gerçek uygulamada HTML-escape yapılmalıdır
        println!(r#"  <input type="hidden" name="{key}" value="{value}" />"#);
    }
    println!(r#"  <input type="submit" value="Ödemeyi Tamamla" />"#);
    println!("</form>");
}
