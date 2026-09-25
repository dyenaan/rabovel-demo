import { Badge } from "@/components/ui/badge";
import type { Blockchain } from "@/types";

const NETWORK_COLOR: Record<Blockchain, string> = {
  ETHEREUM: "bg-[#627EEA]/15 text-[#627EEA]",
  POLYGON: "bg-[#8247E5]/15 text-[#8247E5]",
  ARBITRUM: "bg-[#28A0F0]/15 text-[#28A0F0]",
  BASE: "bg-[#0052FF]/15 text-[#0052FF]",
  SOLANA: "bg-[#14F195]/15 text-[#0f9d63]",
  AVALANCHE: "bg-[#E84142]/15 text-[#E84142]",
};

export function NetworkBadge({ blockchain }: { blockchain: Blockchain }) {
  return (
    <Badge variant="outline" className={`border-transparent capitalize ${NETWORK_COLOR[blockchain]}`}>
      {blockchain.charAt(0) + blockchain.slice(1).toLowerCase()}
    </Badge>
  );
}
