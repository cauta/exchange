import { formatNumberWithCommas } from "@/lib/format";
import type { Token } from "@/lib/types/exchange";
import type { OrderSide, OrderEstimate } from "./types";

interface OrderSummaryProps {
  estimate: OrderEstimate | null;
  side: OrderSide;
  quoteToken: Token;
  priceDecimals: number;
  feeBps: number;
}

export function OrderSummary({ estimate, side, quoteToken, priceDecimals, feeBps }: OrderSummaryProps) {
  if (!estimate || estimate.size <= 0 || estimate.price <= 0) {
    return null;
  }

  const displayDecimals = Math.min(priceDecimals, 4);

  return (
    <div className="space-y-1.5 bg-muted/30 border border-border/40 rounded-md p-3 text-xs">
      <div className="flex justify-between items-center text-muted-foreground">
        <span>Total</span>
        <span className="font-mono">
          {formatNumberWithCommas(estimate.total, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
      <div className="flex justify-between items-center text-muted-foreground">
        <span>Fee ({(Math.abs(feeBps) / 100).toFixed(2)}%)</span>
        <span className="font-mono">
          {formatNumberWithCommas(estimate.fee, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
      <div className="flex justify-between items-center pt-1.5 border-t border-border/40">
        <span className="font-medium">{side === "buy" ? "You Pay" : "You Get"}</span>
        <span className="font-mono font-semibold">
          {formatNumberWithCommas(estimate.finalAmount, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
    </div>
  );
}
