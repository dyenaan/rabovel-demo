pub struct PurchaseTerms<'a> {
    pub investor_wallet: &'a str,
    pub investor_payment_account: &'a str,
    pub payment_mint: &'a str,
    pub payment_token_program: &'a str,
    pub payment_decimals: u8,
    pub broker_payment_account: &'a str,
    pub equity_quantity: u64,
    pub payment_amount: u64,
}
