export type Attestation = {
  attestationId: string;
  attestor: string;
  attestedAt: string;
  verificationStatus: "VERIFIED" | "PENDING" | "FLAGGED";
  reportUrl: string;
};

export const mockAttestations: Attestation[] = [
  {
    attestationId: "att_2025_09",
    attestor: "Halcyon Assurance LLP",
    attestedAt: "2025-09-01",
    verificationStatus: "VERIFIED",
    reportUrl: "#",
  },
  {
    attestationId: "att_2025_08",
    attestor: "Halcyon Assurance LLP",
    attestedAt: "2025-08-01",
    verificationStatus: "VERIFIED",
    reportUrl: "#",
  },
  {
    attestationId: "att_2025_07",
    attestor: "Halcyon Assurance LLP",
    attestedAt: "2025-07-01",
    verificationStatus: "VERIFIED",
    reportUrl: "#",
  },
  {
    attestationId: "att_2025_06",
    attestor: "Halcyon Assurance LLP",
    attestedAt: "2025-06-01",
    verificationStatus: "FLAGGED",
    reportUrl: "#",
  },
];
