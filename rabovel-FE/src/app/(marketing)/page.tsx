import { AssetPreviewGrid } from "./_components/asset-preview-grid";
import { CtaSection } from "./_components/cta-section";
import { HeroSection } from "./_components/hero-section";
import { HowItWorksSteps } from "./_components/how-it-works-steps";
import { InfrastructureGrid } from "./_components/infrastructure-grid";

export default function MarketingHomePage() {
  return (
    <>
      <HeroSection />
      <HowItWorksSteps />
      <AssetPreviewGrid />
      <InfrastructureGrid />
      <CtaSection />
    </>
  );
}
