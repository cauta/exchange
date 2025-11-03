import { formatNumberWithCommas } from "@/lib/format";
import type { Token } from "@/lib/types/openapi";
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
    <div className="space-y-2 bg-gradient-to-br from-muted/40 to-muted/60 border border-border/50 rounded-lg p-4 text-xs shadow-inner">
      <div className="flex justify-between items-center">
        <span className="text-muted-foreground font-medium">Total</span>
        <span className="font-mono font-semibold text-sm">
          {formatNumberWithCommas(estimate.total, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
      <div className="flex justify-between items-center">
        <span className="text-muted-foreground font-medium">
          Est. Fee ({(Math.abs(feeBps) / 100).toFixed(2)}%)
        </span>
        <span className="font-mono text-muted-foreground">
          {formatNumberWithCommas(estimate.fee, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
      <div className="flex justify-between items-center pt-2 border-t border-border/50">
        <span className={`font-semibold ${side === "buy" ? "text-green-600" : "text-red-600"}`}>
          {side === "buy" ? "Total Cost" : "You Receive"}
        </span>
        <span className={`font-mono font-bold text-sm ${side === "buy" ? "text-green-600" : "text-red-600"}`}>
          {formatNumberWithCommas(estimate.finalAmount, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
    </div>
  );
}
