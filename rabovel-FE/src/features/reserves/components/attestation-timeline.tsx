import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { formatDate } from "@/lib/formatters";
import { mockAttestations } from "@/mocks/reserves.mock";

const STATUS_VARIANT = {
  VERIFIED: "success",
  PENDING: "warning",
  FLAGGED: "destructive",
} as const;

export function AttestationTimeline() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Attestation Timeline</CardTitle>
      </CardHeader>
      <CardContent className="pb-6">
        <ul className="space-y-4">
          {mockAttestations.map((attestation) => (
            <li key={attestation.attestationId} className="flex items-start gap-3">
              <div className="mt-1.5 size-2 shrink-0 rounded-full bg-primary" />
              <div className="flex-1">
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium text-foreground">{attestation.attestor}</p>
                  <Badge variant={STATUS_VARIANT[attestation.verificationStatus]}>
                    {attestation.verificationStatus}
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground">{formatDate(attestation.attestedAt)}</p>
              </div>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}
