import { Button } from "@/components/ui/button";
import type { Token } from "@/lib/types/openapi";
import type { OrderSide } from "./types";

interface SubmitButtonProps {
  side: OrderSide;
  baseToken: Token;
  isAuthenticated: boolean;
  loading: boolean;
}

export function SubmitButton({ side, baseToken, isAuthenticated, loading }: SubmitButtonProps) {
  const getButtonText = () => {
    if (loading) return "Placing Order...";
    if (!isAuthenticated) return "Connect Wallet";
    return `${side === "buy" ? "Buy" : "Sell"} ${baseToken.ticker}`;
  };

  return (
    <Button
      type="submit"
      disabled={loading || !isAuthenticated}
      size="lg"
      className={`w-full font-bold text-base h-12 shadow-lg transition-all ${
        side === "buy"
          ? "bg-gradient-to-br from-green-600 to-green-700 hover:from-green-700 hover:to-green-800 text-white shadow-green-600/30 hover:shadow-green-600/50"
          : "bg-gradient-to-br from-red-600 to-red-700 hover:from-red-700 hover:to-red-800 text-white shadow-red-600/30 hover:shadow-red-600/50"
      }`}
    >
      {getButtonText()}
    </Button>
  );
}
