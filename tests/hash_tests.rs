use nestpay::{
    hash::{compute_hash, escape_value, verify_hash},
    CardType, Currency, NestPayRequest, ThreeDPayHostingRequest, ThreeDPayRequest, ThreeDRequest,
    TransactionType,
};

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests — escape_value
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn escape_plain_string_unchanged() {
    assert_eq!(escape_value("hello"), "hello");
    assert_eq!(escape_value("95.93"), "95.93");
    assert_eq!(escape_value(""), "");
}

#[test]
fn escape_pipe_character() {
    assert_eq!(escape_value("a|b"), "a\\|b");
    assert_eq!(escape_value("|"), "\\|");
}

#[test]
fn escape_backslash_character() {
    assert_eq!(escape_value("a\\b"), "a\\\\b");
    assert_eq!(escape_value("\\"), "\\\\");
}

#[test]
fn escape_backslash_then_pipe_order_matters() {
    // "a\|b" must become "a\\|b", NOT "a\\\\|b".
    // Backslash must be escaped FIRST so the newly introduced `\` in `\|`
    // is not double-escaped.
    assert_eq!(escape_value("a\\|b"), "a\\\\\\|b");
}

// ──────────────────────────────────────────────────────────────────────────────
// Unit tests — compute_hash / verify_hash
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn hash_output_is_88_char_base64() {
    // SHA-512 → 64 bytes → base64 → 88 characters (with `==` padding).
    let hash = compute_hash(&[("amount", "95.93")], "TEST1234");
    assert_eq!(hash.len(), 88, "unexpected base64 length: {hash}");
    assert!(
        hash.ends_with('='),
        "SHA-512 base64 must have padding: {hash}"
    );
}

#[test]
fn hash_is_deterministic() {
    let params = vec![("amount", "95.93"), ("clientid", "100200127")];
    assert_eq!(
        compute_hash(&params, "TEST1234"),
        compute_hash(&params, "TEST1234")
    );
}

#[test]
fn hash_differs_for_different_store_keys() {
    let params = vec![("amount", "100.00")];
    assert_ne!(
        compute_hash(&params, "KEY1"),
        compute_hash(&params, "KEY2")
    );
}

#[test]
fn sort_order_does_not_affect_hash() {
    // Params supplied in different orders must produce the same hash.
    let p1 = vec![
        ("clientid", "100200127"),
        ("amount", "95.93"),
        ("currency", "949"),
    ];
    let p2 = vec![
        ("currency", "949"),
        ("amount", "95.93"),
        ("clientid", "100200127"),
    ];
    let p3 = vec![
        ("amount", "95.93"),
        ("currency", "949"),
        ("clientid", "100200127"),
    ];
    assert_eq!(compute_hash(&p1, "K"), compute_hash(&p2, "K"));
    assert_eq!(compute_hash(&p1, "K"), compute_hash(&p3, "K"));
}

#[test]
fn sort_is_case_insensitive_ascii() {
    // "BillToName" (lowercase: "billtoname") sorts between "amount" and "clientid".
    let p1 = vec![
        ("amount", "1"),
        ("BillToName", "X"),
        ("clientid", "2"),
    ];
    let p2 = vec![
        ("BillToName", "X"),
        ("clientid", "2"),
        ("amount", "1"),
    ];
    assert_eq!(compute_hash(&p1, "K"), compute_hash(&p2, "K"));
}

#[test]
fn hash_and_encoding_and_countdown_are_excluded() {
    let base = vec![("amount", "50.00"), ("rnd", "abc")];
    let with_noise = vec![
        ("amount", "50.00"),
        ("rnd", "abc"),
        ("hash", "should_be_ignored"),
        ("HASH", "also_ignored"),
        ("Hash", "also_ignored"),
        ("encoding", "UTF-8"),
        ("ENCODING", "UTF-8"),
        ("countdown", "300"),
        ("Countdown", "60"),
    ];
    assert_eq!(compute_hash(&base, "K"), compute_hash(&with_noise, "K"));
}

#[test]
fn empty_value_included_in_plaintext() {
    // An empty Instalment must produce a different hash than omitting it,
    // because it contributes an empty segment ("||") to the plaintext.
    let with_empty_instalment = vec![
        ("amount", "50.00"),
        ("Instalment", ""),
        ("rnd", "xyz"),
    ];
    let without_instalment = vec![("amount", "50.00"), ("rnd", "xyz")];
    assert_ne!(
        compute_hash(&with_empty_instalment, "K"),
        compute_hash(&without_instalment, "K")
    );
}

#[test]
fn verify_hash_roundtrip() {
    let params = vec![
        ("amount", "95.93"),
        ("clientid", "100200127"),
        ("currency", "949"),
        ("rnd", "1234567890"),
    ];
    let store_key = "TEST1234";
    let hash = compute_hash(&params, store_key);
    assert!(verify_hash(&params, store_key, &hash));
    assert!(!verify_hash(&params, store_key, "wronghash"));
    assert!(!verify_hash(&params, "WRONG_KEY", &hash));
}

// ──────────────────────────────────────────────────────────────────────────────
// Integration test — documentation example (page 6)
//
// Parameters taken from the official NestPay Hash Ver3 documentation (2024).
// The `rnd` field is fixed so the test is deterministic.
//
// To verify against the documentation:
//   1. Run: `cargo test doc_example -- --nocapture`
//   2. Compare the printed hash with the value on page 6 of the PDF.
//
// Alternatively, verify independently via shell:
//   echo -n "<printed_plaintext>" | sha512sum | xxd -r -p | base64
// ──────────────────────────────────────────────────────────────────────────────

#[test]
fn doc_example_3d_hash_computation() {
    // These are the exact field values shown in the documentation example.
    // `rnd` is fixed here; in production it must be unique per transaction.
    let params = vec![
        ("clientid", "100200127"),
        ("amount", "95.93"),
        ("okurl", "https://yoursite.com/ok"),
        ("failUrl", "https://yoursite.com/fail"),
        ("TranType", "Auth"),
        ("Instalment", ""),
        ("callbackUrl", "https://yoursite.com/callback"),
        ("currency", "949"),
        ("rnd", "1715000000"),
        ("storetype", "3d"),
        ("hashAlgorithm", "ver3"),
        ("lang", "tr"),
        ("BillToName", "name"),
        ("BillToCompany", "billToCompany"),
    ];

    let store_key = "TEST1234";
    let hash = compute_hash(&params, store_key);

    // Bağımsız olarak Python ile doğrulandı (PHP natcasesort + sha512 + base64
    // algoritmasının birebir eşdeğeri):
    //
    //   sorted_plaintext = "95.93|billToCompany|name|https://yoursite.com/callback|
    //                        100200127|949|https://yoursite.com/fail|ver3||tr|
    //                        https://yoursite.com/ok|1715000000|3d|Auth|TEST1234"
    const EXPECTED: &str =
        "TX/adVRVplssPyiZuXkvUxlRRroHIvbChDcK2gbi03HkzcopbMVUqkfk8FO0HXuSC7QgRS6t6cnUYE51nKiKbQ==";

    assert_eq!(hash, EXPECTED, "hash does not match verified test vector");
    println!("Dokuman örneği hash: {hash}");
}

// ──────────────────────────────────────────────────────────────────────────────
// Integration tests — model structs
// ──────────────────────────────────────────────────────────────────────────────

fn sample_3d() -> ThreeDRequest {
    ThreeDRequest {
        client_id: "100200127".into(),
        amount: "95.93".into(),
        ok_url: "https://yoursite.com/ok".into(),
        fail_url: "https://yoursite.com/fail".into(),
        callback_url: "https://yoursite.com/callback".into(),
        tran_type: TransactionType::Auth,
        instalment: String::new(),
        currency: Currency::Try,
        rnd: "1715000000".into(),
        lang: "tr".into(),
        pan: "4111111111111111".into(),
        cv2: "000".into(),
        exp_year: "26".into(),
        exp_month: "12".into(),
        card_type: CardType::Visa,
        bill_to_name: Some("name".into()),
        bill_to_company: Some("billToCompany".into()),
    }
}

#[test]
fn three_d_request_storetype_is_3d() {
    let params = sample_3d().to_params();
    let storetype = params.iter().find(|(k, _)| k == "storetype");
    assert_eq!(storetype.map(|(_, v)| v.as_str()), Some("3d"));
}

#[test]
fn three_d_request_has_hash_algorithm_ver3() {
    let params = sample_3d().to_params();
    let algo = params.iter().find(|(k, _)| k == "hashAlgorithm");
    assert_eq!(algo.map(|(_, v)| v.as_str()), Some("ver3"));
}

#[test]
fn three_d_request_contains_card_fields() {
    let params = sample_3d().to_params();
    let keys: Vec<&str> = params.iter().map(|(k, _)| k.as_str()).collect();
    assert!(keys.contains(&"pan"), "pan missing");
    assert!(keys.contains(&"cv2"), "cv2 missing");
    assert!(
        keys.contains(&"Ecom_Payment_Card_ExpDate_Year"),
        "exp year missing"
    );
    assert!(
        keys.contains(&"Ecom_Payment_Card_ExpDate_Month"),
        "exp month missing"
    );
    assert!(keys.contains(&"cardType"), "cardType missing");
}

#[test]
fn three_d_pay_storetype() {
    let req = ThreeDPayRequest {
        client_id: "100200127".into(),
        amount: "95.93".into(),
        ok_url: "https://yoursite.com/ok".into(),
        fail_url: "https://yoursite.com/fail".into(),
        callback_url: "https://yoursite.com/callback".into(),
        tran_type: TransactionType::Auth,
        instalment: String::new(),
        currency: Currency::Try,
        rnd: "1715000000".into(),
        lang: "tr".into(),
        pan: "4111111111111111".into(),
        cv2: "000".into(),
        exp_year: "26".into(),
        exp_month: "12".into(),
        card_type: CardType::Visa,
        bill_to_name: None,
        bill_to_company: None,
    };
    let params = req.to_params();
    let storetype = params.iter().find(|(k, _)| k == "storetype");
    assert_eq!(storetype.map(|(_, v)| v.as_str()), Some("3D_PAY"));
}

#[test]
fn three_d_pay_hosting_storetype_and_no_card_fields() {
    let req = ThreeDPayHostingRequest {
        client_id: "100200127".into(),
        amount: "95.93".into(),
        ok_url: "https://yoursite.com/ok".into(),
        fail_url: "https://yoursite.com/fail".into(),
        callback_url: "https://yoursite.com/callback".into(),
        tran_type: TransactionType::Auth,
        instalment: String::new(),
        currency: Currency::Try,
        rnd: "1715000000".into(),
        lang: "tr".into(),
        bill_to_name: None,
        bill_to_company: None,
        refresh_time: Some("5".into()),
    };
    let params = req.to_params();
    let keys: Vec<&str> = params.iter().map(|(k, _)| k.as_str()).collect();

    let storetype = params.iter().find(|(k, _)| k == "storetype");
    assert_eq!(storetype.map(|(_, v)| v.as_str()), Some("3D_PAY_HOSTING"));

    // No card data in 3D_PAY_HOSTING
    assert!(!keys.contains(&"pan"), "pan must not be present");
    assert!(!keys.contains(&"cv2"), "cv2 must not be present");

    // refreshtime must be present
    let rt = params.iter().find(|(k, _)| k == "refreshtime");
    assert_eq!(rt.map(|(_, v)| v.as_str()), Some("5"));
}

#[test]
fn form_params_includes_hash_field() {
    let req = sample_3d();
    let form = req.form_params("TEST1234");
    let hash_field = form.iter().find(|(k, _)| k == "hash");
    assert!(hash_field.is_some(), "form_params must include 'hash' field");
    assert_eq!(hash_field.unwrap().1.len(), 88);
}

#[test]
fn three_d_compute_hash_matches_raw_compute_hash() {
    let req = sample_3d();
    let store_key = "TEST1234";

    let model_hash = req.compute_hash(store_key);
    let raw_hash = compute_hash(&req.to_params(), store_key);

    assert_eq!(model_hash, raw_hash);
}

#[test]
fn optional_bill_to_fields_absent_when_none() {
    let req = ThreeDRequest {
        bill_to_name: None,
        bill_to_company: None,
        ..sample_3d()
    };
    let params = req.to_params();
    let keys: Vec<&str> = params.iter().map(|(k, _)| k.as_str()).collect();
    assert!(
        !keys.contains(&"BillToName"),
        "BillToName must be absent when None"
    );
    assert!(
        !keys.contains(&"BillToCompany"),
        "BillToCompany must be absent when None"
    );
}

#[test]
fn optional_bill_to_fields_present_when_some() {
    let req = ThreeDRequest {
        bill_to_name: Some("Ad Soyad".into()),
        bill_to_company: Some("Firma".into()),
        ..sample_3d()
    };
    let params = req.to_params();
    let name = params.iter().find(|(k, _)| k == "BillToName");
    let company = params.iter().find(|(k, _)| k == "BillToCompany");
    assert_eq!(name.map(|(_, v)| v.as_str()), Some("Ad Soyad"));
    assert_eq!(company.map(|(_, v)| v.as_str()), Some("Firma"));
}

#[test]
fn card_type_values_correct() {
    let visa = CardType::Visa;
    let mc = CardType::MasterCard;
    assert_eq!(visa.as_str(), "1");
    assert_eq!(mc.as_str(), "2");
}

#[test]
fn currency_codes_correct() {
    assert_eq!(Currency::Try.as_str(), "949");
    assert_eq!(Currency::Usd.as_str(), "840");
    assert_eq!(Currency::Eur.as_str(), "978");
    assert_eq!(Currency::Gbp.as_str(), "826");
    assert_eq!(Currency::Jpy.as_str(), "392");
}

#[test]
fn transaction_type_strings_correct() {
    assert_eq!(TransactionType::Auth.as_str(), "Auth");
    assert_eq!(TransactionType::PreAuth.as_str(), "PreAuth");
    assert_eq!(TransactionType::PostAuth.as_str(), "PostAuth");
    assert_eq!(TransactionType::Credit.as_str(), "Credit");
    assert_eq!(TransactionType::Void.as_str(), "Void");
}
