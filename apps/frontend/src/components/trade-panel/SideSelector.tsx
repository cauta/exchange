import { Button } from "@/components/ui/button";
import type { OrderSide } from "./types";

interface SideSelectorProps {
  value: OrderSide;
  onChange: (value: OrderSide) => void;
}

export function SideSelector({ value, onChange }: SideSelectorProps) {
  return (
    <div className="grid grid-cols-2 gap-3">
      <Button
        onClick={() => onChange("buy")}
        variant={value === "buy" ? "default" : "outline"}
        className={
          value === "buy"
            ? "bg-gradient-to-br from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white shadow-lg shadow-green-600/20 border-green-500/50"
            : "hover:bg-green-600/10 border-green-600/20 hover:border-green-600/40"
        }
        size="lg"
      >
        Buy
      </Button>
      <Button
        onClick={() => onChange("sell")}
        variant={value === "sell" ? "default" : "outline"}
        className={
          value === "sell"
            ? "bg-gradient-to-br from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white shadow-lg shadow-red-600/20 border-red-500/50"
            : "hover:bg-red-600/10 border-red-600/20 hover:border-red-600/40"
        }
        size="lg"
      >
        Sell
      </Button>
    </div>
  );
}
