"use client";

import { Search } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { AssetClass } from "@/types";
import { ASSET_CLASS_LABEL } from "./asset-card";

const ASSET_CLASSES = Object.keys(ASSET_CLASS_LABEL) as AssetClass[];

export function AssetFilters({
  search,
  onSearchChange,
  selectedClass,
  onSelectedClassChange,
}: {
  search: string;
  onSearchChange: (value: string) => void;
  selectedClass: AssetClass | "ALL";
  onSelectedClassChange: (value: AssetClass | "ALL") => void;
}) {
  return (
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div className="relative w-full sm:max-w-xs">
        <Search className="absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          placeholder="Search assets…"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          className="pl-8"
        />
      </div>
      <div className="flex flex-wrap gap-1.5">
        <Button
          size="sm"
          variant={selectedClass === "ALL" ? "default" : "outline"}
          onClick={() => onSelectedClassChange("ALL")}
          className={cn("h-8")}
        >
          All
        </Button>
        {ASSET_CLASSES.map((assetClass) => (
          <Button
            key={assetClass}
            size="sm"
            variant={selectedClass === assetClass ? "default" : "outline"}
            onClick={() => onSelectedClassChange(assetClass)}
            className="h-8"
          >
            {ASSET_CLASS_LABEL[assetClass]}
          </Button>
        ))}
      </div>
    </div>
  );
}
