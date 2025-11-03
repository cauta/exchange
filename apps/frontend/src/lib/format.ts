/**
 * Formatting utilities for market data with proper decimal handling
 */

import { toDisplayValue } from "@exchange/sdk";

// Re-export toDisplayValue for convenience
export { toDisplayValue };

/**
 * Format a number with commas and remove trailing zeros
 * @param value Number to format
 * @param maxDecimals Maximum number of decimal places
 * @param keepTrailingZeros If true, keeps trailing zeros (e.g., for prices)
 * @returns Formatted string with commas
 */
export function formatNumberWithCommas(
  value: number,
  maxDecimals: number = 8,
  keepTrailingZeros: boolean = false
): string {
  // Format with max decimals
  const fixed = value.toFixed(maxDecimals);

  // If we want to keep trailing zeros, don't trim them
  const trimmed = keepTrailingZeros ? fixed : fixed.replace(/\.?0+$/, "");

  // Split into integer and decimal parts
  const parts = trimmed.split(".");
  const integer = parts[0] || "0";
  const decimal = parts[1];

  // Add commas to integer part
  const withCommas = integer.replace(/\B(?=(\d{3})+(?!\d))/g, ",");

  // Rejoin with decimal if it exists
  return decimal !== undefined ? `${withCommas}.${decimal}` : withCommas;
}

/**
 * Round a price to the nearest tick size
 * @param price Display price
 * @param tickSize Raw tick size from market
 * @param quoteDecimals Quote token decimals
 * @returns Rounded display price
 */
export function roundToTickSize(price: number, tickSize: string, quoteDecimals: number): number {
  const tickValue = toDisplayValue(tickSize, quoteDecimals);
  if (tickValue === 0) return price;
  return Math.round(price / tickValue) * tickValue;
}

/**
 * Round a size to the nearest lot size
 * @param size Display size
 * @param lotSize Raw lot size from market
 * @param baseDecimals Base token decimals
 * @returns Rounded display size
 */
export function roundToLotSize(size: number, lotSize: string, baseDecimals: number): number {
  const lotValue = toDisplayValue(lotSize, baseDecimals);
  if (lotValue === 0) return size;
  return Math.round(size / lotValue) * lotValue;
}

/**
 * Get the number of decimal places needed to display a tick/lot size
 * @param tickOrLotSize Raw tick/lot size
 * @param decimals Token decimals
 * @returns Number of decimal places
 */
export function getDecimalPlaces(tickOrLotSize: string, decimals: number): number {
  const displayValue = toDisplayValue(tickOrLotSize, decimals);
  if (displayValue === 0) return decimals;

  const str = displayValue.toFixed(decimals);
  const trimmed = str.replace(/\.?0+$/, "");
  const decimalIndex = trimmed.indexOf(".");

  if (decimalIndex === -1) return 0;
  return trimmed.length - decimalIndex - 1;
}

/**
 * Format a number with a maximum number of decimals, removing trailing zeros
 * @param value Number to format
 * @param maxDecimals Maximum number of decimal places
 * @returns Formatted string without trailing zeros
 */
export function formatWithoutTrailingZeros(value: number, maxDecimals: number): string {
  const fixed = value.toFixed(maxDecimals);
  return parseFloat(fixed).toString();
}
