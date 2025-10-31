"use client";

import { useEffect, useRef } from "react";
import { useExchangeStore } from "@/lib/store";
import { ExchangeDatafeed } from "@/lib/tradingview-datafeed";
import { Card, CardContent } from "@/components/ui/card";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - TradingView types
import type {
  IChartingLibraryWidget,
  ChartingLibraryWidgetOptions,
  ResolutionString,
} from "../../public/vendor/trading-view/charting_library";

// Extend window to include TradingView
declare global {
  interface Window {
    TradingView?: {
      widget: new (options: ChartingLibraryWidgetOptions) => IChartingLibraryWidget;
    };
  }
}

export function TradingViewChart() {
  const containerRef = useRef<HTMLDivElement>(null);
  const widgetRef = useRef<IChartingLibraryWidget | null>(null);
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);

  useEffect(() => {
    if (!containerRef.current || !selectedMarketId) return;

    // Check if TradingView library is loaded
    if (typeof window === "undefined" || !window.TradingView) {
      console.error("TradingView library not loaded");
      return;
    }

    const TradingView = window.TradingView;

    const widgetOptions: ChartingLibraryWidgetOptions = {
      symbol: selectedMarketId,
      datafeed: new ExchangeDatafeed(),
      interval: "1" as ResolutionString, // 1 minute
      container: containerRef.current,
      library_path: "/vendor/trading-view/",
      locale: "en",
      disabled_features: ["use_localstorage_for_settings", "volume_force_overlay"],
      enabled_features: ["study_templates"],
      fullscreen: false,
      autosize: true,
      theme: "dark",
      custom_css_url: "/tradingview-custom.css",
      loading_screen: {
        backgroundColor: "#0d0a14",
        foregroundColor: "#9d7efa",
      },
      settings_overrides: {
        // Background - Match card background (hsl(260, 30%, 8%))
        "paneProperties.background": "#0d0a14",
        "paneProperties.backgroundType": "solid",
        "paneProperties.backgroundGradientStartColor": "#0d0a14",
        "paneProperties.backgroundGradientEndColor": "#0d0a14",

        // Grid lines - subtle purple tint
        "paneProperties.vertGridProperties.color": "#1f1832",
        "paneProperties.horzGridProperties.color": "#1f1832",

        // Separators
        "paneProperties.separatorColor": "#1f1832",

        // Chart style - 1 for candles
        "mainSeriesProperties.style": 1,

        // Candle colors - matching your green/red theme
        "mainSeriesProperties.candleStyle.upColor": "#10b981",
        "mainSeriesProperties.candleStyle.downColor": "#ef4444",
        "mainSeriesProperties.candleStyle.wickUpColor": "#10b981",
        "mainSeriesProperties.candleStyle.wickDownColor": "#ef4444",
        "mainSeriesProperties.candleStyle.borderUpColor": "#10b981",
        "mainSeriesProperties.candleStyle.borderDownColor": "#ef4444",
        "mainSeriesProperties.candleStyle.drawWick": true,
        "mainSeriesProperties.candleStyle.drawBorder": true,

        // Line chart colors (fallback) - primary purple
        "mainSeriesProperties.lineStyle.color": "#9d7efa",
        "mainSeriesProperties.lineStyle.linewidth": 2,

        // Crosshair
        "crosshairProperties.color": "#9d7efa",

        // Scale text color and background
        "scalesProperties.textColor": "#a295c1",
        "scalesProperties.backgroundColor": "#0d0a14",
        "scalesProperties.lineColor": "#1f1832",
      },
      studies_overrides: {
        "volume.volume.color.0": "#ef4444",
        "volume.volume.color.1": "#10b981",
        "volume.volume.transparency": 70,
      },
    };

    try {
      const widget = new TradingView.widget(widgetOptions);
      widgetRef.current = widget;

      widget.onChartReady(() => {
        console.log("TradingView chart is ready");

        // Force candlestick chart style
        widget.activeChart().setChartType(1); // 1 = Candles
      });
    } catch (error) {
      console.error("Failed to create TradingView widget:", error);
    }

    // Cleanup
    return () => {
      if (widgetRef.current) {
        widgetRef.current.remove();
        widgetRef.current = null;
      }
    };
  }, [selectedMarketId]);

  if (!selectedMarketId) {
    return (
      <Card className="flex items-center justify-center h-full min-h-[400px]">
        <CardContent className="p-6">
          <p className="text-gray-500 text-sm">Select a market to view chart</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="overflow-hidden h-full p-0">
      <div ref={containerRef} className="h-full w-full" />
    </Card>
  );
}
