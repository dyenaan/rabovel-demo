import type { UserRole } from "@/types";

export type AuthUser = {
  id: string;
  name: string;
  email: string;
  role: UserRole;
  investorId?: string;
  avatarUrl?: string;
  roles?: UserRole[];
  onboardingStatus?: "approved" | "requires_review" | "blocked";
};
