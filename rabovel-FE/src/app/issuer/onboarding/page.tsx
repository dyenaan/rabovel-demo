"use client";

import { useState, type ReactNode } from "react";
import { useRouter } from "next/navigation";
import { ArrowLeft, ArrowRight, Building2, Check, Loader2, ShieldCheck, UploadCloud, UserRound } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Progress } from "@/components/ui/progress";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { submitIssuerOnboarding, type IssuerOnboardingSubmission } from "@/features/auth/api/backend-auth";
import { cn } from "@/lib/utils";
import { useAuthStore } from "@/stores/auth-store";

const steps = [
  { label: "Organization", icon: Building2 },
  { label: "Representative", icon: UserRound },
  { label: "Review", icon: ShieldCheck },
];

const initialForm: IssuerOnboardingSubmission = {
  legal_name: "",
  organization_type: "",
  registration_number: "",
  jurisdiction: "",
  registered_address: "",
  representative_name: "",
  representative_title: "",
  document_reference: "",
  beneficial_owners_confirmed: false,
  information_certified: false,
};

export default function IssuerOnboardingPage() {
  const router = useRouter();
  const token = useAuthStore((state) => state.token);
  const user = useAuthStore((state) => state.user);
  const [step, setStep] = useState(0);
  const [form, setForm] = useState(initialForm);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const setField = <K extends keyof IssuerOnboardingSubmission>(key: K, value: IssuerOnboardingSubmission[K]) =>
    setForm((current) => ({ ...current, [key]: value }));

  const organizationComplete = [form.legal_name, form.organization_type, form.registration_number, form.jurisdiction, form.registered_address].every((value) => value.trim().length >= 2);
  const representativeComplete = [form.representative_name, form.representative_title, form.document_reference].every((value) => value.trim().length >= 2) && form.beneficial_owners_confirmed;

  async function submit() {
    if (!token) return;
    setIsSubmitting(true);
    try {
      await submitIssuerOnboarding(token, form);
      toast.success("Organization verified for this demo.");
      router.push("/issuer");
      router.refresh();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Could not submit organization verification.");
    } finally {
      setIsSubmitting(false);
    }
  }

  return (
    <div className="mx-auto max-w-3xl space-y-6">
      <div>
        <p className="text-sm font-medium text-primary">Organization verification</p>
        <h1 className="text-3xl font-semibold tracking-tight">Issuer onboarding</h1>
        <p className="mt-2 text-muted-foreground">A short mock KYB flow for your organization.</p>
      </div>

      <div className="space-y-3">
        <div className="flex justify-between">
          {steps.map(({ label, icon: Icon }, index) => (
            <div key={label} className={cn("flex items-center gap-2 text-xs font-medium sm:text-sm", index <= step ? "text-primary" : "text-muted-foreground")}>
              <span className={cn("flex size-7 items-center justify-center rounded-full border", index < step && "border-primary bg-primary text-primary-foreground")}>
                {index < step ? <Check className="size-4" /> : <Icon className="size-4" />}
              </span>
              <span className="hidden sm:inline">{label}</span>
            </div>
          ))}
        </div>
        <Progress value={((step + 1) / steps.length) * 100} />
      </div>

      {step === 0 && (
        <Card>
          <CardHeader><CardTitle>Organization details</CardTitle><CardDescription>Enter the legal information shown on your registration documents.</CardDescription></CardHeader>
          <CardContent className="grid gap-4 sm:grid-cols-2">
            <Field label="Legal organization name" className="sm:col-span-2"><Input value={form.legal_name} onChange={(event) => setField("legal_name", event.target.value)} autoFocus /></Field>
            <Field label="Organization type">
              <Select value={form.organization_type} onValueChange={(value) => setField("organization_type", value)}>
                <SelectTrigger className="w-full"><SelectValue placeholder="Select entity type" /></SelectTrigger>
                <SelectContent><SelectItem value="Corporation">Corporation</SelectItem><SelectItem value="Limited partnership">Limited partnership</SelectItem><SelectItem value="Limited liability company">Limited liability company</SelectItem><SelectItem value="Trust">Trust</SelectItem></SelectContent>
              </Select>
            </Field>
            <Field label="Registration number"><Input value={form.registration_number} onChange={(event) => setField("registration_number", event.target.value)} /></Field>
            <Field label="Jurisdiction"><Input value={form.jurisdiction} onChange={(event) => setField("jurisdiction", event.target.value)} placeholder="e.g. Abuja, Nigeria" /></Field>
            <Field label="Registered address" className="sm:col-span-2"><Input value={form.registered_address} onChange={(event) => setField("registered_address", event.target.value)} /></Field>
          </CardContent>
        </Card>
      )}

      {step === 1 && (
        <Card>
          <CardHeader><CardTitle>Authorized representative</CardTitle><CardDescription>Confirm who is completing onboarding for the organization.</CardDescription></CardHeader>
          <CardContent className="grid gap-4 sm:grid-cols-2">
            <Field label="Full legal name"><Input value={form.representative_name} onChange={(event) => setField("representative_name", event.target.value)} placeholder={user?.name} autoFocus /></Field>
            <Field label="Title or position"><Input value={form.representative_title} onChange={(event) => setField("representative_title", event.target.value)} placeholder="e.g. Director" /></Field>
            <div className="space-y-2 sm:col-span-2">
              <Label>Registration document</Label>
              <button type="button" onClick={() => setField("document_reference", "certificate-of-incorporation.pdf")} className="flex w-full flex-col items-center gap-2 rounded-lg border border-dashed p-7 text-center transition-colors hover:bg-muted/40">
                {form.document_reference ? <Check className="size-6 text-success" /> : <UploadCloud className="size-6 text-muted-foreground" />}
                <span className="text-sm font-medium">{form.document_reference || "Upload certificate of incorporation"}</span>
                <span className="text-xs text-muted-foreground">Demo upload — no file leaves your browser</span>
              </button>
            </div>
            <label className="flex items-start gap-3 rounded-lg border p-4 sm:col-span-2">
              <Checkbox checked={form.beneficial_owners_confirmed} onCheckedChange={(value) => setField("beneficial_owners_confirmed", value === true)} className="mt-0.5" />
              <span className="text-sm leading-relaxed">I confirm that the organization has identified its beneficial owners and controlling persons.</span>
            </label>
          </CardContent>
        </Card>
      )}

      {step === 2 && (
        <Card>
          <CardHeader><CardTitle>Review and certify</CardTitle><CardDescription>This demo approves the organization immediately after submission.</CardDescription></CardHeader>
          <CardContent className="space-y-5">
            <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-3 rounded-lg bg-muted/40 p-4 text-sm">
              <dt className="text-muted-foreground">Organization</dt><dd className="font-medium">{form.legal_name}</dd>
              <dt className="text-muted-foreground">Entity</dt><dd>{form.organization_type}</dd>
              <dt className="text-muted-foreground">Registration</dt><dd>{form.registration_number}</dd>
              <dt className="text-muted-foreground">Jurisdiction</dt><dd>{form.jurisdiction}</dd>
              <dt className="text-muted-foreground">Representative</dt><dd>{form.representative_name}, {form.representative_title}</dd>
              <dt className="text-muted-foreground">Document</dt><dd>{form.document_reference}</dd>
            </dl>
            <label className="flex items-start gap-3 rounded-lg border p-4">
              <Checkbox checked={form.information_certified} onCheckedChange={(value) => setField("information_certified", value === true)} className="mt-0.5" />
              <span className="text-sm leading-relaxed">I certify that this information is accurate and that I am authorized to act for the organization.</span>
            </label>
          </CardContent>
        </Card>
      )}

      <div className="flex items-center justify-between">
        <Button variant="outline" onClick={() => step === 0 ? router.push("/issuer") : setStep((current) => current - 1)}><ArrowLeft className="size-4" />{step === 0 ? "Cancel" : "Back"}</Button>
        {step < 2 ? (
          <Button disabled={step === 0 ? !organizationComplete : !representativeComplete} onClick={() => setStep((current) => current + 1)}>Continue <ArrowRight className="size-4" /></Button>
        ) : (
          <Button disabled={!form.information_certified || isSubmitting} onClick={submit}>{isSubmitting ? <Loader2 className="size-4 animate-spin" /> : <ShieldCheck className="size-4" />}{isSubmitting ? "Verifying…" : "Submit for verification"}</Button>
        )}
      </div>
    </div>
  );
}

function Field({ label, className, children }: { label: string; className?: string; children: ReactNode }) {
  return <div className={cn("space-y-2", className)}><Label>{label}</Label>{children}</div>;
}
