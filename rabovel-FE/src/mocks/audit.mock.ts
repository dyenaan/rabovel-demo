export type AuditLogEntry = {
  logId: string;
  actor: string;
  action: string;
  target: string;
  timestamp: string;
  ipAddress: string;
};

export const mockAuditLogs: AuditLogEntry[] = [
  {
    logId: "log_1",
    actor: "admin@rabovel.com",
    action: "APPROVED_KYC",
    target: "inv_1003 (Amara Okafor)",
    timestamp: new Date(Date.now() - 1000 * 60 * 30).toISOString(),
    ipAddress: "203.0.113.24",
  },
  {
    logId: "log_2",
    actor: "compliance@rabovel.com",
    action: "OPENED_COMPLIANCE_CASE",
    target: "inv_1004 (Julian Voss)",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 5).toISOString(),
    ipAddress: "203.0.113.31",
  },
  {
    logId: "log_3",
    actor: "system",
    action: "SETTLEMENT_FINALIZED",
    target: "stl_5003",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 4).toISOString(),
    ipAddress: "internal",
  },
  {
    logId: "log_4",
    actor: "admin@rabovel.com",
    action: "SUSPENDED_ACCOUNT",
    target: "inv_1005 (Priya Raman)",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 60).toISOString(),
    ipAddress: "203.0.113.24",
  },
  {
    logId: "log_5",
    actor: "issuer@rabovel.com",
    action: "CREATED_ASSET_DRAFT",
    target: "Pre-IPO Growth Equity Fund",
    timestamp: new Date(Date.now() - 1000 * 60 * 60 * 24 * 90).toISOString(),
    ipAddress: "198.51.100.12",
  },
];
