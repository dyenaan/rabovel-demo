export type KycStatus =
  | "NOT_STARTED"
  | "PENDING"
  | "VERIFIED"
  | "REJECTED"
  | "REQUIRES_ACTION"
  | "EXPIRED";

export type EligibilityStatus =
  | "NOT_ASSESSED"
  | "ELIGIBLE"
  | "INELIGIBLE"
  | "RESTRICTED"
  | "PENDING_REVIEW";

export type AccountStatus = "ACTIVE" | "SUSPENDED" | "CLOSED" | "PENDING";

export type AccountType = "INDIVIDUAL" | "INSTITUTION" | "ENTITY";

export type Investor = {
  investorId: string;
  fullName: string;
  email: string;
  accountType: AccountType;
  jurisdiction: string;
  kycStatus: KycStatus;
  eligibilityStatus: EligibilityStatus;
  accountStatus: AccountStatus;
  createdAt: string;
  avatarUrl?: string;
};

export type ComplianceStatus =
  | "PENDING"
  | "VERIFIED"
  | "REJECTED"
  | "REQUIRES_ACTION"
  | "RESTRICTED";
