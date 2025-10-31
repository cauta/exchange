"use client";

import { useState, useEffect } from "react";
import { useTurnkey } from "@turnkey/sdk-react";
import { useExchangeStore } from "@/lib/store";
import { autoFaucet } from "@/lib/faucet";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function AuthButton() {
  const { turnkey, passkeyClient } = useTurnkey();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const setUser = useExchangeStore((state) => state.setUser);
  const clearUser = useExchangeStore((state) => state.clearUser);
  const tokens = useExchangeStore((state) => state.tokens);

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [email, setEmail] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Check if Turnkey is configured
  const isTurnkeyConfigured = turnkey && process.env.NEXT_PUBLIC_TURNKEY_ORGANIZATION_ID;

  // Try to restore session on mount
  useEffect(() => {
    const checkSession = async () => {
      if (!isTurnkeyConfigured) return;

      try {
        // Try to get current session from Turnkey
        const session = await turnkey.getCurrentUser?.();
        if (session?.userId) {
          // Use the user ID as the address for now
          // In production, you'd get the wallet address from Turnkey
          setUser(session.userId);

          // Auto-faucet for new users
          if (tokens.length > 0) {
            await autoFaucet(session.userId, tokens);
          }
        }
      } catch (err) {
        console.log("No existing session");
      }
    };
    checkSession();
  }, [isTurnkeyConfigured, turnkey, setUser, tokens]);

  const handleEmailAuth = async () => {
    if (!email.trim()) {
      setError("Please enter your email");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      if (!turnkey) {
        throw new Error("Turnkey not initialized");
      }

      // Create or login with email
      const result = await turnkey.login({
        organizationId: process.env.NEXT_PUBLIC_TURNKEY_ORGANIZATION_ID!,
        email: email.trim(),
      });

      if (result?.userId) {
        setUser(result.userId);
        setIsDialogOpen(false);
        setEmail("");

        // Auto-faucet for new users
        if (tokens.length > 0) {
          await autoFaucet(result.userId, tokens);
        }
      }
    } catch (err) {
      console.error("Auth error:", err);
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setIsLoading(false);
    }
  };

  const handlePasskeyAuth = async () => {
    setIsLoading(true);
    setError(null);

    try {
      if (!passkeyClient) {
        throw new Error("Passkey client not initialized");
      }

      const result = await passkeyClient.login();

      if (result?.userId) {
        setUser(result.userId);
        setIsDialogOpen(false);

        // Auto-faucet for new users
        if (tokens.length > 0) {
          await autoFaucet(result.userId, tokens);
        }
      }
    } catch (err) {
      console.error("Passkey auth error:", err);
      setError(err instanceof Error ? err.message : "Passkey authentication failed");
    } finally {
      setIsLoading(false);
    }
  };

  const handleDemoConnect = () => {
    // Fallback demo mode if Turnkey not configured
    const demoAddress = `user_${Date.now()}`;
    setUser(demoAddress);
    setIsDialogOpen(false);
    setError(null);

    // Auto-faucet for demo users
    if (tokens.length > 0) {
      autoFaucet(demoAddress, tokens);
    }
  };

  const handleLogout = () => {
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
    <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
      <DialogTrigger asChild>
        <Button size="sm" variant="default">
          Connect Wallet
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Connect Your Wallet</DialogTitle>
          <DialogDescription>
            {isTurnkeyConfigured
              ? "Sign in with email or passkey to create or access your embedded wallet"
              : "Demo mode - Auto-generate a wallet to try the exchange"}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {isTurnkeyConfigured ? (
            <>
              {/* Email Authentication */}
              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <div className="flex gap-2">
                  <Input
                    id="email"
                    type="email"
                    placeholder="your@email.com"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleEmailAuth();
                    }}
                    disabled={isLoading}
                  />
                  <Button onClick={handleEmailAuth} disabled={isLoading}>
                    {isLoading ? "Sending..." : "Continue"}
                  </Button>
                </div>
              </div>

              <div className="relative">
                <div className="absolute inset-0 flex items-center">
                  <span className="w-full border-t" />
                </div>
                <div className="relative flex justify-center text-xs uppercase">
                  <span className="bg-background px-2 text-muted-foreground">Or</span>
                </div>
              </div>

              {/* Passkey Authentication */}
              <Button
                onClick={handlePasskeyAuth}
                disabled={isLoading}
                variant="outline"
                className="w-full"
              >
                Sign in with Passkey
              </Button>
            </>
          ) : (
            <>
              {/* Demo Mode */}
              <div className="bg-yellow-500/10 border border-yellow-500/20 p-3 text-sm text-yellow-600 dark:text-yellow-500 rounded-md">
                <p className="font-semibold mb-1">Demo Mode</p>
                <p className="text-xs">
                  Turnkey is not configured. A demo wallet will be created with 10,000 tokens of each type.
                </p>
              </div>

              <Button onClick={handleDemoConnect} className="w-full">
                Create Demo Wallet
              </Button>

              <div className="text-xs text-muted-foreground text-center pt-2">
                To enable real embedded wallets, add your Turnkey organization ID to .env.local
              </div>
            </>
          )}

          {error && (
            <div className="rounded-md bg-destructive/15 p-3 text-sm text-destructive">
              {error}
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
