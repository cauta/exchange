import type { ExchangeDatafeed } from "@/lib/tradingview-datafeed";

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
  datafeed: ExchangeDatafeed,
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
    theme: "dark",
    custom_css_url: "/tradingview-custom.css",
    loading_screen: {
      backgroundColor: "#1a1a1a",
      foregroundColor: "#ffffff",
    },
    overrides: {
      // Background
      "paneProperties.background": "#1a1a1a",
      "paneProperties.backgroundType": "solid",
      "paneProperties.vertGridProperties.color": "#262626",
      "paneProperties.horzGridProperties.color": "#262626",
      "paneProperties.vertGridProperties.style": 0,
      "paneProperties.horzGridProperties.style": 0,

      // Candle colors
      "mainSeriesProperties.candleStyle.upColor": "#22c55e",
      "mainSeriesProperties.candleStyle.downColor": "#ef4444",
      "mainSeriesProperties.candleStyle.wickUpColor": "#22c55e",
      "mainSeriesProperties.candleStyle.wickDownColor": "#ef4444",
      "mainSeriesProperties.candleStyle.borderUpColor": "#22c55e",
      "mainSeriesProperties.candleStyle.borderDownColor": "#ef4444",
      "mainSeriesProperties.candleStyle.drawWick": true,
      "mainSeriesProperties.candleStyle.drawBorder": true,

      // Hollow candles
      "mainSeriesProperties.hollowCandleStyle.upColor": "#22c55e",
      "mainSeriesProperties.hollowCandleStyle.downColor": "#ef4444",
      "mainSeriesProperties.hollowCandleStyle.wickUpColor": "#22c55e",
      "mainSeriesProperties.hollowCandleStyle.wickDownColor": "#ef4444",
      "mainSeriesProperties.hollowCandleStyle.borderUpColor": "#22c55e",
      "mainSeriesProperties.hollowCandleStyle.borderDownColor": "#ef4444",

      // Bars
      "mainSeriesProperties.barStyle.upColor": "#22c55e",
      "mainSeriesProperties.barStyle.downColor": "#ef4444",

      // Line
      "mainSeriesProperties.lineStyle.color": "#ffffff",
      "mainSeriesProperties.lineStyle.linewidth": 2,

      // Area
      "mainSeriesProperties.areaStyle.color1": "rgba(255, 255, 255, 0.1)",
      "mainSeriesProperties.areaStyle.color2": "rgba(255, 255, 255, 0.02)",
      "mainSeriesProperties.areaStyle.linecolor": "#ffffff",
      "mainSeriesProperties.areaStyle.linewidth": 2,

      // Baseline
      "mainSeriesProperties.baselineStyle.topLineColor": "#22c55e",
      "mainSeriesProperties.baselineStyle.bottomLineColor": "#ef4444",

      // Scales and axes
      "scalesProperties.backgroundColor": "#1a1a1a",
      "scalesProperties.lineColor": "#262626",
      "scalesProperties.textColor": "#ffffff",

      // Crosshair
      "crosshairProperties.color": "#ffffff",
      "crosshairProperties.width": 1,
      "crosshairProperties.style": 2,

      // Watermark
      "paneProperties.legendProperties.showLegend": true,
      "paneProperties.legendProperties.showStudyArguments": true,
      "paneProperties.legendProperties.showStudyTitles": true,
      "paneProperties.legendProperties.showStudyValues": true,
      "paneProperties.legendProperties.showSeriesTitle": true,
      "paneProperties.legendProperties.showSeriesOHLC": true,
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
