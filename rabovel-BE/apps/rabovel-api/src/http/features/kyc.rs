use axum::{routing::post, Router};

use crate::{kyc_webhook_handler, submit_kyc_case_handler, GatewayState};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/kyc/cases", post(submit_kyc_case_handler))
        .route("/kyc/webhook", post(kyc_webhook_handler))
}
