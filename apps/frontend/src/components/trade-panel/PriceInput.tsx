import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toDisplayValue, roundToTickSize, getDecimalPlaces } from "@/lib/format";
import type { Token, Market } from "@/lib/types/openapi";

interface PriceInputProps {
  value: string;
  onChange: (value: string) => void;
  market: Market;
  quoteToken: Token;
  error?: string;
}

export function PriceInput({ value, onChange, market, quoteToken, error }: PriceInputProps) {
  const priceDecimals = getDecimalPlaces(market.tick_size, quoteToken.decimals);

  const handleBlur = () => {
    if (value) {
      const numPrice = parseFloat(value);
      if (!isNaN(numPrice) && numPrice > 0) {
        const rounded = roundToTickSize(numPrice, market.tick_size, quoteToken.decimals);
        onChange(rounded.toFixed(priceDecimals));
      }
    }
  };

  return (
    <div className="space-y-2">
      <Label className="text-xs font-semibold text-muted-foreground uppercase tracking-wide">
        Price ({quoteToken.ticker})
      </Label>
      <Input
        type="number"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={handleBlur}
        placeholder="0.00"
        step={toDisplayValue(market.tick_size, quoteToken.decimals)}
        className={`font-mono h-11 text-base border-border/50 focus:border-primary/50 focus:ring-primary/20 bg-muted/30 ${
          error ? "border-red-500/50 focus:border-red-500/50 focus:ring-red-500/20" : ""
        }`}
      />
      {error && <p className="text-xs text-red-600 font-medium">{error}</p>}
    </div>
  );
}
