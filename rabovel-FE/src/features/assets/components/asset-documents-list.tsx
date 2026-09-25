import { Download, FileText } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { EmptyState } from "@/components/shared/empty-state";
import { formatDate } from "@/lib/formatters";
import type { AssetDocument } from "@/types";

function formatFileSize(bytes: number): string {
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function AssetDocumentsList({ documents }: { documents?: AssetDocument[] }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Documents</CardTitle>
      </CardHeader>
      <CardContent className="pb-6">
        {!documents || documents.length === 0 ? (
          <EmptyState icon={FileText} title="No documents" description="Offering documents will appear here once published." />
        ) : (
          <ul className="divide-y">
            {documents.map((doc) => (
              <li key={doc.documentId} className="flex items-center justify-between py-3 first:pt-0 last:pb-0">
                <div className="flex items-center gap-3">
                  <div className="flex size-9 items-center justify-center rounded-md bg-muted">
                    <FileText className="size-4 text-muted-foreground" />
                  </div>
                  <div>
                    <p className="text-sm font-medium text-foreground">{doc.title}</p>
                    <p className="text-xs text-muted-foreground">
                      {doc.fileType} · {formatFileSize(doc.sizeBytes)} · {formatDate(doc.publishedAt)}
                    </p>
                  </div>
                </div>
                <Button variant="ghost" size="icon" aria-label={`Download ${doc.title}`}>
                  <Download className="size-4" />
                </Button>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  );
}
