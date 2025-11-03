"use client";

import { useState } from "react";
import { useExchangeStore } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import { toRawValue } from "@/lib/format";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { toast } from "sonner";
import { useTurnkey } from "@turnkey/react-wallet-kit";
import { Droplet } from "lucide-react";

export function FaucetDialog() {
  const client = useExchangeClient();
  const { handleLogin } = useTurnkey();
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const [open, setOpen] = useState(false);
  const [loadingToken, setLoadingToken] = useState<string | null>(null);

  const handleFaucet = async (tokenTicker: string) => {
    if (!userAddress) {
      return;
    }

    setLoadingToken(tokenTicker);

    try {
      const token = tokens.find((t) => t.ticker === tokenTicker);
      if (!token) {
        throw new Error("Token not found");
      }

      // Faucet 1000 tokens (adjust amount with decimals)
      const amount = toRawValue(1000, token.decimals);

      await client.rest.faucet({
        userAddress,
        tokenTicker,
        amount,
        signature: `${userAddress}:${Date.now()}`,
      });

      toast.success(`Successfully received 1000 ${tokenTicker}!`, {
        description: "Your balance has been updated",
      });
    } catch (err) {
      console.error("Faucet error:", err);
      toast.error("Failed to get tokens", {
        description: err instanceof Error ? err.message : "Please try again later",
      });
    } finally {
      setLoadingToken(null);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" variant="outline" className="gap-2">
          <Droplet className="h-4 w-4" />
          Faucet
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md data-[state=open]:slide-in-from-bottom-4 data-[state=closed]:slide-out-to-bottom-4">
        <DialogHeader>
          <DialogTitle className="text-xl">Token Faucet</DialogTitle>
          <DialogDescription>
            {isAuthenticated
              ? "Select a token to receive 1000 tokens for testing"
              : "Connect your wallet to use the faucet"}
          </DialogDescription>
        </DialogHeader>

        {!isAuthenticated ? (
          <div className="flex flex-col items-center gap-4 py-6">
            <p className="text-sm text-muted-foreground text-center">
              You need to connect your wallet before you can use the faucet
            </p>
            <Button
              onClick={() => {
                setOpen(false);
                handleLogin();
              }}
              className="backdrop-blur-md bg-primary/80 hover:bg-primary/90"
            >
              Connect Wallet
            </Button>
          </div>
        ) : (
          <div className="grid gap-3 py-4">
            {tokens.map((token) => (
              <div
                key={token.ticker}
                className="flex items-center justify-between p-4 rounded-lg border border-border bg-gradient-to-br from-card/50 to-muted/30 hover:from-card/70 hover:to-muted/40 hover:border-primary/30 transition-all duration-200 shadow-sm hover:shadow-md"
              >
                <div className="flex items-center gap-3">
                  <div className="w-10 h-10 rounded-full bg-primary/10 border border-primary/20 flex items-center justify-center">
                    <Droplet className="h-5 w-5 text-primary" />
                  </div>
                  <div>
                    <div className="font-bold text-base">{token.ticker}</div>
                    <div className="text-xs text-muted-foreground">{token.name}</div>
                  </div>
                </div>
                <Button
                  size="sm"
                  onClick={() => handleFaucet(token.ticker)}
                  disabled={loadingToken !== null}
                  className="bg-primary hover:bg-primary/90 shadow-sm hover:shadow-md transition-all"
                >
                  {loadingToken === token.ticker ? "Getting..." : "Get 1000"}
                </Button>
              </div>
            ))}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
