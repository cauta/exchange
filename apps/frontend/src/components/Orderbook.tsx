"use client";

import { useOrderbook } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { formatPrice, formatSize } from "@/lib/format";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const { bids, asks } = useOrderbook(selectedMarketId);

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500">Select a market to view orderbook</p>
      </div>
    );
  }

  // Look up token decimals
  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500">Loading token information...</p>
      </div>
    );
  }

  return (
    <div className="p-4 border rounded flex flex-col h-full">
      <h3 className="text-lg font-bold mb-4">Orderbook</h3>

      <div className="flex justify-between font-bold mb-2 text-sm text-gray-500 px-2">
        <span>Price ({quoteToken.ticker})</span>
        <span>Size ({baseToken.ticker})</span>
      </div>

      <div className="flex-1 overflow-y-auto min-h-0">
        {/* Asks (Sell orders - Red) - Top section */}
        <div className="flex flex-col-reverse px-2">
          {asks.map((ask, i) => (
            <div key={i} className="flex justify-between text-sm text-red-500 py-0.5">
              <span>{formatPrice(ask.price, quoteToken.decimals)}</span>
              <span>{formatSize(ask.size, baseToken.decimals)}</span>
            </div>
          ))}
        </div>

        {/* Spread separator */}
        <div className="border-t border-gray-700 my-2"></div>

        {/* Bids (Buy orders - Green) - Bottom section */}
        <div className="px-2">
          {bids.map((bid, i) => (
            <div key={i} className="flex justify-between text-sm text-green-500 py-0.5">
              <span>{formatPrice(bid.price, quoteToken.decimals)}</span>
              <span>{formatSize(bid.size, baseToken.decimals)}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
