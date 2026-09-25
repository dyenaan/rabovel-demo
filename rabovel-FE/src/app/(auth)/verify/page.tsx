import type { Metadata } from "next";

import { VerifyForm } from "@/features/auth/components/verify-form";

export const metadata: Metadata = { title: "Verify Email" };

export default function VerifyPage() {
  return <VerifyForm />;
}
