//! Örnek: **3D_PAY_HOSTING** modeli ile ödeme isteği oluşturma
//!
//! Bu modelde banka kart giriş sayfasını barındırır; merchant tarafında
//! kart numarası, CVV veya son kullanma tarihi gerekmez.
//!
//! Çalıştırmak için:
//! ```
//! cargo run --example three_d_pay_hosting_payment
//! ```

use nestpay::{Currency, NestPayRequest, ThreeDPayHostingRequest, TransactionType};

fn main() {
    let request = ThreeDPayHostingRequest {
        client_id: "100200127".into(),
        amount: "91.96".into(),
        ok_url: "https://yoursite.com/ok".into(),
        fail_url: "https://yoursite.com/fail".into(),
        callback_url: "https://yoursite.com/callback".into(),
        tran_type: TransactionType::Auth,
        instalment: String::new(),
        currency: Currency::Try,
        rnd: "0.56435200 1715000000".into(),
        lang: "tr".into(),
        bill_to_name: Some("Ad Soyad".into()),
        bill_to_company: None,
        refresh_time: Some("5".into()), // 5 saniye sonra otomatik yönlendir
    };

    let store_key = "TEST1234";
    let form_params = request.form_params(store_key);

    println!("--- 3D_PAY_HOSTING parametreleri ---");
    for (key, value) in &form_params {
        println!("{key:<40} = {value}");
    }

    println!("\n3D_PAY_HOSTING modeli: kart bilgisi formu yok, banka yönlendiriyor.");
}
