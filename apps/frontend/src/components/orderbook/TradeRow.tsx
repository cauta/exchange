"use client";

interface TradeRowProps {
  price: string;
  size: string;
  time: string;
  side: "buy" | "sell";
}

export function TradeRow({ price, size, time, side }: TradeRowProps) {
  const colorClass = side === "buy" ? "text-green-500" : "text-red-500";

  return (
    <div className="flex justify-between items-center text-[11px] leading-tight hover:bg-muted/50 px-3 py-0.5 font-mono tabular-nums">
      <span className={`${colorClass} font-semibold`}>{price}</span>
      <span className="text-muted-foreground text-right">{size}</span>
      <span className="text-muted-foreground text-[10px] text-right">{time}</span>
    </div>
  );
}
