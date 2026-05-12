//! Örnek: **3D_PAY** modeli ile ödeme isteği oluşturma
//!
//! Çalıştırmak için:
//! ```
//! cargo run --example three_d_pay_payment
//! ```

use nestpay::{CardType, Currency, NestPayRequest, ThreeDPayRequest, TransactionType};

fn main() {
    let request = ThreeDPayRequest {
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
        pan: "5100000000000008".into(),
        cv2: "000".into(),
        exp_year: "26".into(),
        exp_month: "12".into(),
        card_type: CardType::MasterCard,
        bill_to_name: Some("Ad Soyad".into()),
        bill_to_company: None,
    };

    let store_key = "TEST1234";
    let form_params = request.form_params(store_key);

    println!("--- 3D_PAY parametreleri ---");
    for (key, value) in &form_params {
        println!("{key:<40} = {value}");
    }

    println!(
        "\nstoretype: {}",
        form_params
            .iter()
            .find(|(k, _)| k == "storetype")
            .map(|(_, v)| v.as_str())
            .unwrap_or("?")
    );
}
