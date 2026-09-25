import { Check, CircleDashed, Coins, FilePenLine, PackageCheck, ShieldCheck, Store } from "lucide-react";

const stages = [
  { label: "Draft", detail: "Define terms & metadata", icon: FilePenLine },
  { label: "Review", detail: "Instant demo approval", icon: ShieldCheck },
  { label: "Mint setup", detail: "Token-2022 & controls", icon: Coins },
  { label: "Inventory", detail: "Backing & issuance", icon: PackageCheck },
  { label: "Listed", detail: "Ready for investors", icon: Store },
];

export function AssetLifecycle({ current = 0 }: { current?: number }) {
  return <div className="overflow-x-auto pb-2"><ol className="grid min-w-[720px] grid-cols-5">
    {stages.map((stage, index) => { const Icon = stage.icon; const complete = index < current; const active = index === current; return <li key={stage.label} className="relative pr-3">
      {index < stages.length - 1 && <div className={`absolute left-10 right-0 top-5 h-px ${complete ? "bg-primary" : "bg-border"}`} />}
      <div className="relative flex items-start gap-3"><div className={`z-10 flex size-10 shrink-0 items-center justify-center rounded-full border bg-card ${active ? "border-primary text-primary ring-4 ring-primary/10" : complete ? "border-primary bg-primary text-primary-foreground" : "text-muted-foreground"}`}>{complete ? <Check className="size-4" /> : active ? <Icon className="size-4" /> : <CircleDashed className="size-4" />}</div><div className="pt-0.5"><p className={`text-sm font-medium ${active ? "text-primary" : !complete ? "text-muted-foreground" : ""}`}>{stage.label}</p><p className="mt-0.5 text-xs text-muted-foreground">{stage.detail}</p>{active && <span className="mt-2 inline-flex rounded-full bg-primary/10 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-primary">Current stage</span>}</div></div>
    </li>; })}
  </ol></div>;
}

export function lifecycleStep(status: string) {
  if (status === "pending_review") return 1;
  if (status === "approved_for_setup") return 2;
  if (status === "minted") return 3;
  return 0;
}
