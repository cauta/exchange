// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import type {
  ChartingLibraryWidgetOptions,
  ResolutionString,
} from "../../../public/vendor/trading-view/charting_library";

/**
 * TradingView chart configuration
 * Centralized configuration for theme, colors, and behavior
 */
export function getChartConfig(
  symbol: string,
  datafeed: any,
  container: HTMLElement
): ChartingLibraryWidgetOptions {
  return {
    symbol,
    datafeed,
    interval: "1" as ResolutionString,
    container,
    library_path: "/vendor/trading-view/",
    locale: "en",
    disabled_features: [
      "use_localstorage_for_settings",
      "volume_force_overlay",
      "header_symbol_search",
      "symbol_search_hot_key",
    ],
    enabled_features: ["study_templates", "side_toolbar_in_fullscreen_mode"],
    fullscreen: false,
    autosize: true,
    theme: "Dark",
    custom_css_url: "/tradingview-custom.css",
    loading_screen: {
      backgroundColor: "#1a1a1a",
      foregroundColor: "#9d7efa",
    },
    settings_overrides: {
      // Background
      "paneProperties.background": "#1a1a1a",
      "paneProperties.backgroundType": "solid",

      // Chart style - candlestick
      "mainSeriesProperties.style": 1,

      // Candle colors
      "mainSeriesProperties.candleStyle.upColor": "#22c55e",
      "mainSeriesProperties.candleStyle.downColor": "#ef4444",
      "mainSeriesProperties.candleStyle.wickUpColor": "#22c55e",
      "mainSeriesProperties.candleStyle.wickDownColor": "#ef4444",
      "mainSeriesProperties.candleStyle.borderUpColor": "#22c55e",
      "mainSeriesProperties.candleStyle.borderDownColor": "#ef4444",
      "mainSeriesProperties.candleStyle.drawWick": true,
      "mainSeriesProperties.candleStyle.drawBorder": true,

      // Crosshair
      "crosshairProperties.color": "#9d7efa",
      "crosshairProperties.width": 1,
      "crosshairProperties.style": 2,

      // Scales
      "scalesProperties.backgroundColor": "#1a1a1a",
    },
    studies_overrides: {
      "volume.volume.color.0": "#ef4444",
      "volume.volume.color.1": "#22c55e",
      "volume.volume.transparency": 65,
    },
    time_frames: [
      { text: "1m", resolution: "1" as ResolutionString, description: "1 Minute" },
      { text: "5m", resolution: "5" as ResolutionString, description: "5 Minutes" },
      { text: "15m", resolution: "15" as ResolutionString, description: "15 Minutes" },
      { text: "1h", resolution: "60" as ResolutionString, description: "1 Hour" },
      { text: "4h", resolution: "240" as ResolutionString, description: "4 Hours" },
      { text: "1D", resolution: "D" as ResolutionString, description: "1 Day" },
    ],
  };
}
