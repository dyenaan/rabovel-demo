use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    cngn_payment_asset_handler, confirm_asset_image_upload_handler, create_asset_draft_handler,
    create_asset_image_upload_handler, create_broker_cngn_account_handler, get_asset_draft_handler,
    get_asset_setup_operation_handler, get_initial_inventory_handler,
    issue_initial_inventory_handler, issuer_onboarding_handler, issuer_overview_handler,
    list_asset_drafts_handler, publish_asset_listing_handler, start_asset_setup_handler,
    submit_asset_backing_handler, submit_asset_for_demo_review_handler, update_asset_draft_handler,
    GatewayState,
};

pub(crate) fn routes() -> Router<GatewayState> {
    Router::new()
        .route("/issuer/overview", get(issuer_overview_handler))
        .route("/issuer/onboarding", post(issuer_onboarding_handler))
        .route(
            "/issuer/assets",
            get(list_asset_drafts_handler).post(create_asset_draft_handler),
        )
        .route(
            "/issuer/assets/:asset_id/image-upload",
            post(create_asset_image_upload_handler),
        )
        .route(
            "/issuer/assets/:asset_id/image-upload/confirm",
            post(confirm_asset_image_upload_handler),
        )
        .route(
            "/issuer/assets/:asset_id",
            get(get_asset_draft_handler).patch(update_asset_draft_handler),
        )
        .route(
            "/issuer/assets/:asset_id/submit",
            post(submit_asset_for_demo_review_handler),
        )
        .route(
            "/issuer/assets/:asset_id/setup",
            get(get_asset_setup_operation_handler).post(start_asset_setup_handler),
        )
        .route(
            "/issuer/assets/:asset_id/inventory",
            get(get_initial_inventory_handler).post(issue_initial_inventory_handler),
        )
        .route(
            "/issuer/assets/:asset_id/backing",
            post(submit_asset_backing_handler),
        )
        .route(
            "/issuer/assets/:asset_id/listing",
            post(publish_asset_listing_handler),
        )
        .route(
            "/payment-assets/cngn",
            get(cngn_payment_asset_handler).post(create_broker_cngn_account_handler),
        )
}
