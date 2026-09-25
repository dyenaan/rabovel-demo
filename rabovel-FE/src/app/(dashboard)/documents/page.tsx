import type { Metadata } from "next";

import { PageContainer } from "@/components/layout/page-container";
import { PageHeader } from "@/components/shared/page-header";
import { AssetDocumentsList } from "@/features/assets/components/asset-documents-list";
import { mockAssets } from "@/mocks/assets.mock";

export const metadata: Metadata = { title: "Documents" };

export default function DocumentsPage() {
  const documents = mockAssets.flatMap((asset) =>
    (asset.documents ?? []).map((doc) => ({ ...doc, title: `${doc.title} — ${asset.symbol}` })),
  );

  return (
    <PageContainer>
      <PageHeader
        title="Documents"
        description="Offering documents, audits, and reports for the assets in your portfolio."
      />
      <div className="max-w-3xl">
        <AssetDocumentsList documents={documents} />
      </div>
    </PageContainer>
  );
}
