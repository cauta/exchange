/**
 * Global state management with Zustand
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";
import type { Market, Token, Orderbook, Trade, OrderbookLevel, Balance, Order, OrderStatus } from "./types/exchange";

// ============================================================================
// State Interface
// ============================================================================

interface ExchangeState {
  // Market data
  markets: Market[];
  tokens: Token[];

  // UI Data
  selectedMarketId: string | null;
  orderbook: Orderbook | null;
  recentTrades: Trade[];
  selectedPrice: number | null;

  // User Data
  userAddress: string | null;
  isAuthenticated: boolean;
  userBalances: Balance[];
  userOrders: Order[];
  userTrades: Trade[];

  // Actions - Market Data
  setMarkets: (markets: Market[]) => void;
  setTokens: (tokens: Token[]) => void;

  // Actions - UI Data
  selectMarket: (marketId: string) => void;
  setSelectedPrice: (price: number | null) => void;
  updateOrderbook: (marketId: string, bids: OrderbookLevel[], asks: OrderbookLevel[]) => void;
  addTrade: (trade: Trade) => void;

  // Actions - User Data
  setUser: (address: string) => void;
  clearUser: () => void;
  setBalances: (balances: Balance[]) => void;
  updateBalance: (tokenTicker: string, available: string, locked: string) => void;
  setOrders: (orders: Order[]) => void;
  updateOrder: (orderId: string, status: OrderStatus, filledSize: string) => void;
  setUserTrades: (trades: Trade[]) => void;
  addUserTrade: (trade: Trade) => void;

  // Utilities
  reset: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState = {
  // Market Data
  markets: [],
  tokens: [],

  // UI Data
  selectedMarketId: null,
  orderbook: null,
  recentTrades: [],
  selectedPrice: null,

  // User Data
  userAddress: null,
  isAuthenticated: false,
  userBalances: [],
  userOrders: [],
  userTrades: [],
};

// ============================================================================
// Store
// ============================================================================

export const useExchangeStore = create<ExchangeState>()(
  devtools(
    immer((set) => ({
      ...initialState,

      // ========================================================================
      // Market Data Actions
      // ========================================================================

      setMarkets: (markets) =>
        set((state) => {
          state.markets = markets;
        }),

      setTokens: (tokens) =>
        set((state) => {
          state.tokens = tokens;
        }),

      // ========================================================================
      // UI Data Actions
      // ========================================================================

      selectMarket: (marketId) =>
        set((state) => {
          state.selectedMarketId = marketId;
          state.orderbook = null;
          state.recentTrades = [];
        }),

      setSelectedPrice: (price) =>
        set((state) => {
          state.selectedPrice = price;
        }),

      updateOrderbook: (marketId, bids, asks) =>
        set((state) => {
          if (state.selectedMarketId === marketId) {
            state.orderbook = {
              market_id: marketId,
              bids,
              asks,
              timestamp: Date.now(),
            };
          }
        }),

      addTrade: (trade) =>
        set((state) => {
          if (state.selectedMarketId === trade.market_id) {
            state.recentTrades.unshift(trade);
            if (state.recentTrades.length > 100) {
              state.recentTrades = state.recentTrades.slice(0, 100);
            }
          }
        }),

      // ========================================================================
      // User Data Actions
      // ========================================================================

      setUser: (address) =>
        set((state) => {
          state.userAddress = address;
          state.isAuthenticated = true;
        }),

      clearUser: () =>
        set((state) => {
          state.userAddress = null;
          state.isAuthenticated = false;
          state.userBalances = [];
          state.userOrders = [];
          state.userTrades = [];
        }),

      setBalances: (balances) =>
        set((state) => {
          state.userBalances = balances;
        }),

      updateBalance: (tokenTicker, available, locked) =>
        set((state) => {
          const existingIndex = state.userBalances.findIndex((b) => b.token_ticker === tokenTicker);

          if (existingIndex >= 0 && state.userBalances[existingIndex]) {
            const existing = state.userBalances[existingIndex];
            const totalAmount = (BigInt(available) + BigInt(locked)).toString();
            const token = state.tokens.find((t) => t.ticker === tokenTicker);
            if (!token) return;

            const divisor = Math.pow(10, token.decimals);
            const amountValue = Number(BigInt(totalAmount)) / divisor;
            const lockedValue = Number(BigInt(locked)) / divisor;

            state.userBalances = state.userBalances.map((balance, index) =>
              index === existingIndex
                ? {
                    token_ticker: existing.token_ticker,
                    user_address: existing.user_address,
                    amount: totalAmount,
                    open_interest: locked,
                    updated_at: new Date(),
                    amountDisplay: amountValue.toFixed(token.decimals),
                    lockedDisplay: lockedValue.toFixed(token.decimals),
                    amountValue,
                    lockedValue,
                  }
                : balance
            );
          }
        }),

      setOrders: (orders) =>
        set((state) => {
          state.userOrders = orders;
        }),

      updateOrder: (orderId, status, filledSize) =>
        set((state) => {
          const existingIndex = state.userOrders.findIndex((o) => o.id === orderId);

          if (existingIndex >= 0) {
            const existing = state.userOrders[existingIndex];
            if (!existing) return;

            const market = state.markets.find((m) => m.id === existing.market_id);
            if (!market) return;

            const baseToken = state.tokens.find((t) => t.ticker === market.base_ticker);
            if (!baseToken) return;

            const divisor = Math.pow(10, baseToken.decimals);
            const filledValue = Number(BigInt(filledSize)) / divisor;

            state.userOrders = state.userOrders.map((order, index) =>
              index === existingIndex
                ? {
                    ...existing,
                    status,
                    filled_size: filledSize,
                    filledDisplay: filledValue.toFixed(baseToken.decimals),
                    filledValue,
                  }
                : order
            );
          }
        }),

      setUserTrades: (trades) =>
        set((state) => {
          state.userTrades = trades;
        }),

      addUserTrade: (trade) =>
        set((state) => {
          state.userTrades.unshift(trade);
          if (state.userTrades.length > 100) {
            state.userTrades = state.userTrades.slice(0, 100);
          }
        }),

      // ========================================================================
      // Utilities
      // ========================================================================

      reset: () => set(initialState),
    })),
    { name: "ExchangeStore" }
  )
);

// ============================================================================
// Selectors (for optimized re-renders)
// ============================================================================

// Stable empty arrays to prevent unnecessary re-renders
const EMPTY_ARRAY: OrderbookLevel[] = [];

export const selectSelectedMarket = (state: ExchangeState) =>
  state.markets.find((m) => m.id === state.selectedMarketId);

export const selectOrderbookBids = (state: ExchangeState) => state.orderbook?.bids ?? EMPTY_ARRAY;

export const selectOrderbookAsks = (state: ExchangeState) => state.orderbook?.asks ?? EMPTY_ARRAY;

export const selectRecentTrades = (state: ExchangeState) => state.recentTrades;
