"use client";

import { useEffect } from "react";
import { useTurnkey } from "@turnkey/react-wallet-kit";
import { useExchangeStore } from "@/lib/store";
import { autoFaucet } from "@/lib/faucet";
import { Button } from "@/components/ui/button";

export function AuthButton() {
  const { handleLogin, wallets, authState, logout } = useTurnkey();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const setUser = useExchangeStore((state) => state.setUser);
  const clearUser = useExchangeStore((state) => state.clearUser);
  const tokens = useExchangeStore((state) => state.tokens);

  // Sync Turnkey auth state with our store
  useEffect(() => {
    if (authState === "authenticated" && wallets.length > 0 && !isAuthenticated) {
      // User is authenticated in Turnkey but not in our store
      const firstWallet = wallets[0];
      if (firstWallet && firstWallet.accounts && firstWallet.accounts.length > 0) {
        const address = firstWallet.accounts[0]?.address;
        if (!address) return;
        setUser(address);

        // Auto-faucet for users
        if (tokens.length > 0) {
          autoFaucet(address, tokens);
        }
      }
    } else if (authState === "unauthenticated" && isAuthenticated) {
      // User logged out from Turnkey, sync our store
      clearUser();
    }
  }, [authState, wallets, isAuthenticated, setUser, clearUser, tokens]);

  const handleLogout = () => {
    // Call Turnkey logout and clear local state
    logout();
    clearUser();
  };

  if (isAuthenticated && userAddress) {
    return (
      <div className="flex items-center gap-3">
        <div className="text-sm">
          <span className="text-muted-foreground">Connected: </span>
          <span className="font-mono text-xs">
            {userAddress.slice(0, 6)}...{userAddress.slice(-4)}
          </span>
        </div>
        <Button size="sm" variant="outline" onClick={handleLogout}>
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <Button
      size="sm"
      variant="default"
      className="backdrop-blur-md bg-primary/80 hover:bg-primary/90 border-b-[3px] border-b-primary shadow-[0_3px_2px_0px_rgba(180,150,255,0.6),0_1px_1px_0px_rgba(255,255,255,0.5)] cursor-pointer"
      onClick={() => handleLogin()}
    >
      Connect Wallet
    </Button>
  );
}
