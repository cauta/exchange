"use client";

import { ThemeProvider } from "./theme-provider";
import { TurnkeyProviderWrapper } from "./turnkey-provider";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false} disableTransitionOnChange>
      <TurnkeyProviderWrapper>{children}</TurnkeyProviderWrapper>
    </ThemeProvider>
  );
}
