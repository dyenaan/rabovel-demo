import { mockDelay } from "@/lib/api/mock-delay";
import type { AuthUser } from "../types/auth.types";

const MOCK_USERS: Record<string, { password: string; user: AuthUser }> = {
  "investor@rabovel.com": {
    password: "password123",
    user: {
      id: "usr_investor_1",
      name: "Eleanor Whitfield",
      email: "investor@rabovel.com",
      role: "INVESTOR",
      investorId: "inv_1001",
    },
  },
  "issuer@rabovel.com": {
    password: "password123",
    user: {
      id: "usr_issuer_1",
      name: "Rabovel Demo Issuer",
      email: "issuer@rabovel.com",
      role: "ISSUER",
    },
  },
  "admin@rabovel.com": {
    password: "password123",
    user: {
      id: "usr_admin_1",
      name: "Daniel Ferreira",
      email: "admin@rabovel.com",
      role: "ADMIN",
    },
  },
  "compliance@rabovel.com": {
    password: "password123",
    user: {
      id: "usr_compliance_1",
      name: "Sana Malik",
      email: "compliance@rabovel.com",
      role: "COMPLIANCE",
    },
  },
};

export async function mockLogin(email: string, password: string): Promise<AuthUser> {
  await mockDelay(600);
  const record = MOCK_USERS[email.toLowerCase()];
  if (!record || record.password !== password) {
    throw new Error("Invalid email or password.");
  }
  return record.user;
}

export async function mockRegister(input: {
  fullName: string;
  email: string;
}): Promise<AuthUser> {
  await mockDelay(700);
  return {
    id: `usr_${Date.now()}`,
    name: input.fullName,
    email: input.email,
    role: "INVESTOR",
    investorId: `inv_${Date.now()}`,
  };
}
