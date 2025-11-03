import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { OrderType } from "./types";

interface OrderTypeSelectorProps {
  value: OrderType;
  onChange: (value: OrderType) => void;
}

export function OrderTypeSelector({ value, onChange }: OrderTypeSelectorProps) {
  return (
    <Tabs
      value={value}
      onValueChange={(v) => onChange(v as OrderType)}
      className="w-full"
    >
      <TabsList className="w-full justify-start rounded-none border-b border-border h-auto p-0 bg-card/50 backdrop-blur-sm">
        <TabsTrigger value="limit" className="flex-1 rounded-none">
          Limit
        </TabsTrigger>
        <TabsTrigger value="market" className="flex-1 rounded-none">
          Market
        </TabsTrigger>
      </TabsList>
    </Tabs>
  );
}
