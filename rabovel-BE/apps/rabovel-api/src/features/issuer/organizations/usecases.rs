use crate::app::errors::ApplicationError;
use crate::app::support::unix_now;
use crate::features::identity::access::actor::AuthenticatedUser;
use crate::features::issuer::organizations::dto::IssuerOnboardingRequest;
use crate::features::issuer::organizations::dto::IssuerOrganizationResponse;
use crate::features::issuer::organizations::dto::IssuerOverviewResponse;
use identity::AuthError;
use identity::UserRole;
use issuer_assets::organizations::models::IssuerOrganization;

pub(crate) async fn issuer_overview(
    state: &impl Ports,
    user: AuthenticatedUser,
) -> Result<IssuerOverviewResponse, ApplicationError> {
    let record = state
        .account_queries()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    if !record.principal.has_role(UserRole::Issuer) {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    let organization = state
        .organizations()
        .get_issuer_organization(&user.user_id)
        .await
        .map_err(ApplicationError::from)?;
    Ok(IssuerOverviewResponse {
        user_id: user.user_id,
        email: record.profile.email().to_string(),
        account_status: if organization.is_some() {
            "approved"
        } else {
            "organization_onboarding_pending"
        }
        .to_string(),
        organization: organization.map(issuer_organization_response),
    })
}

pub(crate) fn issuer_organization_response(
    organization: IssuerOrganization,
) -> IssuerOrganizationResponse {
    IssuerOrganizationResponse {
        legal_name: organization.legal_name,
        organization_type: organization.organization_type,
        registration_number: organization.registration_number,
        jurisdiction: organization.jurisdiction,
        registered_address: organization.registered_address,
        representative_name: organization.representative_name,
        representative_title: organization.representative_title,
        document_reference: organization.document_reference,
        approved_at: organization.approved_at,
    }
}

pub(crate) async fn issuer_onboarding(
    state: &impl Ports,
    user: AuthenticatedUser,
    request: IssuerOnboardingRequest,
) -> Result<IssuerOverviewResponse, ApplicationError> {
    let record = state
        .account_queries()
        .get_user(&user.user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    if !record.principal.has_role(UserRole::Issuer) {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }

    let fields = [
        &request.legal_name,
        &request.organization_type,
        &request.registration_number,
        &request.jurisdiction,
        &request.registered_address,
        &request.representative_name,
        &request.representative_title,
        &request.document_reference,
    ];
    if fields.iter().any(|value| value.trim().len() < 2)
        || !request.beneficial_owners_confirmed
        || !request.information_certified
    {
        return Err(ApplicationError::from(AuthError::PolicyViolation(
            "complete every organization field and confirm both attestations".to_string(),
        )));
    }

    let organization = IssuerOrganization {
        user_id: user.user_id.clone(),
        legal_name: request.legal_name.trim().to_string(),
        organization_type: request.organization_type.trim().to_string(),
        registration_number: request.registration_number.trim().to_string(),
        jurisdiction: request.jurisdiction.trim().to_string(),
        registered_address: request.registered_address.trim().to_string(),
        representative_name: request.representative_name.trim().to_string(),
        representative_title: request.representative_title.trim().to_string(),
        document_reference: request.document_reference.trim().to_string(),
        approved_at: unix_now(),
    };
    state
        .organizations()
        .save_issuer_organization(organization.clone())
        .await
        .map_err(ApplicationError::from)?;

    Ok(IssuerOverviewResponse {
        user_id: user.user_id,
        email: record.profile.email().to_string(),
        account_status: "approved".to_string(),
        organization: Some(issuer_organization_response(organization)),
    })
}

pub(crate) async fn require_approved_issuer(
    state: &impl Ports,
    user_id: &str,
) -> Result<(), ApplicationError> {
    let record = state
        .account_queries()
        .get_user(user_id)
        .await
        .map_err(ApplicationError::from)?
        .ok_or_else(|| ApplicationError::from(AuthError::Unauthorized))?;
    if !record.principal.has_role(UserRole::Issuer) {
        return Err(ApplicationError::from(AuthError::ForbiddenAction));
    }
    if state
        .organizations()
        .get_issuer_organization(user_id)
        .await
        .map_err(ApplicationError::from)?
        .is_none()
    {
        return Err(ApplicationError::from(AuthError::OnboardingIncomplete));
    }
    Ok(())
}
pub(crate) trait Ports:
    crate::features::identity::accounts::ports::AccountLookup
    + crate::features::issuer::organizations::ports::OrganizationStore
{
}
impl<
        T: crate::features::identity::accounts::ports::AccountLookup
            + crate::features::issuer::organizations::ports::OrganizationStore
            + ?Sized,
    > Ports for T
{
}
