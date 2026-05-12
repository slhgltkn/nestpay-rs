use crate::hash;

// ──────────────────────────────────────────────────────────────────────────────
// Domain types
// ──────────────────────────────────────────────────────────────────────────────

/// ISO 4217 currency codes accepted by NestPay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    /// Turkish Lira — 949
    Try,
    /// US Dollar — 840
    Usd,
    /// Euro — 978
    Eur,
    /// British Pound — 826
    Gbp,
    /// Japanese Yen — 392
    Jpy,
}

impl Currency {
    pub fn as_str(self) -> &'static str {
        match self {
            Currency::Try => "949",
            Currency::Usd => "840",
            Currency::Eur => "978",
            Currency::Gbp => "826",
            Currency::Jpy => "392",
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// NestPay transaction types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    /// Standard sale / authorization.
    Auth,
    /// Pre-authorization (reserve funds without capture).
    PreAuth,
    /// Capture a previously pre-authorized amount.
    PostAuth,
    /// Refund to a card.
    Credit,
    /// Cancel / void a previously authorized transaction.
    Void,
}

impl TransactionType {
    pub fn as_str(self) -> &'static str {
        match self {
            TransactionType::Auth => "Auth",
            TransactionType::PreAuth => "PreAuth",
            TransactionType::PostAuth => "PostAuth",
            TransactionType::Credit => "Credit",
            TransactionType::Void => "Void",
        }
    }
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Card network / type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardType {
    Visa,
    MasterCard,
}

impl CardType {
    pub fn as_str(self) -> &'static str {
        match self {
            CardType::Visa => "1",
            CardType::MasterCard => "2",
        }
    }
}

impl std::fmt::Display for CardType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Shared trait
// ──────────────────────────────────────────────────────────────────────────────

/// Implemented by all three NestPay payment models.
pub trait NestPayRequest {
    /// Returns every parameter that will be sent to the payment gateway
    /// (excluding the `hash` field itself).
    fn to_params(&self) -> Vec<(String, String)>;

    /// Computes the Ver3 hash for this request.
    fn compute_hash(&self, store_key: &str) -> String {
        hash::compute_hash(&self.to_params(), store_key)
    }

    /// Returns all form parameters **including** the computed `hash` field,
    /// ready to be POST-ed to the NestPay 3D-gate URL.
    ///
    /// # Example
    /// ```no_run
    /// use nestpay::models::{ThreeDRequest, NestPayRequest, TransactionType, Currency, CardType};
    ///
    /// let req = ThreeDRequest {
    ///     client_id:    "100200127".into(),
    ///     amount:       "95.93".into(),
    ///     ok_url:       "https://example.com/ok".into(),
    ///     fail_url:     "https://example.com/fail".into(),
    ///     callback_url: "https://example.com/callback".into(),
    ///     tran_type:    TransactionType::Auth,
    ///     instalment:   String::new(),
    ///     currency:     Currency::Try,
    ///     rnd:          "1234567890".into(),
    ///     lang:         "tr".into(),
    ///     pan:          "4111111111111111".into(),
    ///     cv2:          "000".into(),
    ///     exp_year:     "26".into(),
    ///     exp_month:    "12".into(),
    ///     card_type:    CardType::Visa,
    ///     bill_to_name:    None,
    ///     bill_to_company: None,
    /// };
    ///
    /// let params = req.form_params("YOUR_STORE_KEY");
    /// // POST `params` to the 3D-gate URL
    /// ```
    fn form_params(&self, store_key: &str) -> Vec<(String, String)> {
        let mut params = self.to_params();
        params.push(("hash".to_string(), self.compute_hash(store_key)));
        params
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Shared helper — avoids repeating the common fields in every impl
// ──────────────────────────────────────────────────────────────────────────────

struct CommonArgs<'a> {
    client_id: &'a str,
    amount: &'a str,
    ok_url: &'a str,
    fail_url: &'a str,
    tran_type: TransactionType,
    instalment: &'a str,
    callback_url: &'a str,
    currency: Currency,
    rnd: &'a str,
    lang: &'a str,
    storetype: &'a str,
    bill_to_name: Option<&'a str>,
    bill_to_company: Option<&'a str>,
}

fn common_params(a: CommonArgs<'_>) -> Vec<(String, String)> {
    let mut p = vec![
        ("clientid".to_string(), a.client_id.to_string()),
        ("amount".to_string(), a.amount.to_string()),
        ("okurl".to_string(), a.ok_url.to_string()),
        ("failUrl".to_string(), a.fail_url.to_string()),
        ("TranType".to_string(), a.tran_type.to_string()),
        ("Instalment".to_string(), a.instalment.to_string()),
        ("callbackUrl".to_string(), a.callback_url.to_string()),
        ("currency".to_string(), a.currency.to_string()),
        ("rnd".to_string(), a.rnd.to_string()),
        ("lang".to_string(), a.lang.to_string()),
        ("storetype".to_string(), a.storetype.to_string()),
        ("hashAlgorithm".to_string(), "ver3".to_string()),
    ];
    if let Some(name) = a.bill_to_name {
        p.push(("BillToName".to_string(), name.to_string()));
    }
    if let Some(company) = a.bill_to_company {
        p.push(("BillToCompany".to_string(), company.to_string()));
    }
    p
}

// ──────────────────────────────────────────────────────────────────────────────
// Model: 3D
// The merchant collects card data on their own page and handles the 3DS flow.
// ──────────────────────────────────────────────────────────────────────────────

/// Payment request for the **3D** model (`storetype = "3d"`).
///
/// The merchant's page collects the card number, CVV and expiry, then
/// POSTs them together with the computed hash to the NestPay 3D-gate.
#[derive(Debug, Clone)]
pub struct ThreeDRequest {
    pub client_id: String,
    pub amount: String,
    pub ok_url: String,
    pub fail_url: String,
    pub callback_url: String,
    pub tran_type: TransactionType,
    /// Number of installments, or empty string for a single payment.
    pub instalment: String,
    pub currency: Currency,
    /// A unique random / timestamp value per transaction (e.g. `Uuid::new_v4().to_string()`).
    pub rnd: String,
    /// `"tr"` or `"en"`.
    pub lang: String,
    /// 16-digit PAN (card number).
    pub pan: String,
    /// CVV2 / CVC2.
    pub cv2: String,
    /// Last two digits of expiry year, e.g. `"26"`.
    pub exp_year: String,
    /// Zero-padded expiry month, e.g. `"06"`.
    pub exp_month: String,
    pub card_type: CardType,
    pub bill_to_name: Option<String>,
    pub bill_to_company: Option<String>,
}

impl NestPayRequest for ThreeDRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut p = common_params(CommonArgs {
            client_id: &self.client_id,
            amount: &self.amount,
            ok_url: &self.ok_url,
            fail_url: &self.fail_url,
            tran_type: self.tran_type,
            instalment: &self.instalment,
            callback_url: &self.callback_url,
            currency: self.currency,
            rnd: &self.rnd,
            lang: &self.lang,
            storetype: "3d",
            bill_to_name: self.bill_to_name.as_deref(),
            bill_to_company: self.bill_to_company.as_deref(),
        });
        p.push(("pan".to_string(), self.pan.clone()));
        p.push(("cv2".to_string(), self.cv2.clone()));
        p.push((
            "Ecom_Payment_Card_ExpDate_Year".to_string(),
            self.exp_year.clone(),
        ));
        p.push((
            "Ecom_Payment_Card_ExpDate_Month".to_string(),
            self.exp_month.clone(),
        ));
        p.push(("cardType".to_string(), self.card_type.to_string()));
        p
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Model: 3D_PAY
// Card data collected by merchant; payment is processed entirely by the bank.
// ──────────────────────────────────────────────────────────────────────────────

/// Payment request for the **3D_PAY** model (`storetype = "3D_PAY"`).
///
/// Like `ThreeDRequest` but the bank handles the actual charge after
/// the 3DS authentication redirect.
#[derive(Debug, Clone)]
pub struct ThreeDPayRequest {
    pub client_id: String,
    pub amount: String,
    pub ok_url: String,
    pub fail_url: String,
    pub callback_url: String,
    pub tran_type: TransactionType,
    pub instalment: String,
    pub currency: Currency,
    pub rnd: String,
    pub lang: String,
    pub pan: String,
    pub cv2: String,
    pub exp_year: String,
    pub exp_month: String,
    pub card_type: CardType,
    pub bill_to_name: Option<String>,
    pub bill_to_company: Option<String>,
}

impl NestPayRequest for ThreeDPayRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut p = common_params(CommonArgs {
            client_id: &self.client_id,
            amount: &self.amount,
            ok_url: &self.ok_url,
            fail_url: &self.fail_url,
            tran_type: self.tran_type,
            instalment: &self.instalment,
            callback_url: &self.callback_url,
            currency: self.currency,
            rnd: &self.rnd,
            lang: &self.lang,
            storetype: "3D_PAY",
            bill_to_name: self.bill_to_name.as_deref(),
            bill_to_company: self.bill_to_company.as_deref(),
        });
        p.push(("pan".to_string(), self.pan.clone()));
        p.push(("cv2".to_string(), self.cv2.clone()));
        p.push((
            "Ecom_Payment_Card_ExpDate_Year".to_string(),
            self.exp_year.clone(),
        ));
        p.push((
            "Ecom_Payment_Card_ExpDate_Month".to_string(),
            self.exp_month.clone(),
        ));
        p.push(("cardType".to_string(), self.card_type.to_string()));
        p
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Model: 3D_PAY_HOSTING
// The bank hosts the payment page — no card data needed on the merchant side.
// ──────────────────────────────────────────────────────────────────────────────

/// Payment request for the **3D_PAY_HOSTING** model (`storetype = "3D_PAY_HOSTING"`).
///
/// NestPay hosts the card-entry page.  The merchant only provides transaction
/// metadata and redirects the customer to the bank.
#[derive(Debug, Clone)]
pub struct ThreeDPayHostingRequest {
    pub client_id: String,
    pub amount: String,
    pub ok_url: String,
    pub fail_url: String,
    pub callback_url: String,
    pub tran_type: TransactionType,
    pub instalment: String,
    pub currency: Currency,
    pub rnd: String,
    pub lang: String,
    pub bill_to_name: Option<String>,
    pub bill_to_company: Option<String>,
    /// Seconds before the hosted page auto-redirects (optional).
    pub refresh_time: Option<String>,
}

impl NestPayRequest for ThreeDPayHostingRequest {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut p = common_params(CommonArgs {
            client_id: &self.client_id,
            amount: &self.amount,
            ok_url: &self.ok_url,
            fail_url: &self.fail_url,
            tran_type: self.tran_type,
            instalment: &self.instalment,
            callback_url: &self.callback_url,
            currency: self.currency,
            rnd: &self.rnd,
            lang: &self.lang,
            storetype: "3D_PAY_HOSTING",
            bill_to_name: self.bill_to_name.as_deref(),
            bill_to_company: self.bill_to_company.as_deref(),
        });
        if let Some(ref t) = self.refresh_time {
            p.push(("refreshtime".to_string(), t.clone()));
        }
        p
    }
}
