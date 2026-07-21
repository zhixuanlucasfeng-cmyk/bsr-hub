"use client";

import {
  createContext,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import type { Session, SupabaseClient, User } from "@supabase/supabase-js";
import { ApiClientError, apiFetch } from "../lib/api-client";
import { getAuthConfig, isAuthConfigured } from "../lib/auth-config";
import { createBrowserSupabaseClient } from "../lib/supabase-browser";

export type BsrProfile = {
  id: string;
  authUserId: string;
  displayName: string;
  avatarUrl: string | null;
  role: "buyer" | "seller" | "runner" | "admin";
  trustLevel: number;
};

type AuthContextValue = {
  configured: boolean;
  apiOnline: boolean;
  loading: boolean;
  user: User | null;
  profile: BsrProfile | null;
  accessToken: string | null;
  signIn: (email: string, password: string) => Promise<void>;
  signUp: (email: string, password: string) => Promise<void>;
  signOut: () => Promise<void>;
  bootstrapProfile: () => Promise<void>;
};

const AuthContext = createContext<AuthContextValue | null>(null);

export function AuthProvider({ children }: { children: ReactNode }) {
  const config = getAuthConfig();
  const configured = isAuthConfigured(config);
  const supabase = useMemo<SupabaseClient | null>(() => createBrowserSupabaseClient(), []);
  const [session, setSession] = useState<Session | null>(null);
  const [profile, setProfile] = useState<BsrProfile | null>(null);
  const [apiOnline, setApiOnline] = useState(true);
  const [loading, setLoading] = useState(configured);

  async function bootstrapProfile(nextSession = session) {
    if (!nextSession?.access_token) return;
    try {
      const nextProfile = await apiFetch<BsrProfile>(
        "/v1/profile/bootstrap",
        nextSession.access_token,
        { method: "POST" },
      );
      setProfile(nextProfile);
      setApiOnline(true);
    } catch (error) {
      if (error instanceof ApiClientError) setApiOnline(false);
      throw error;
    }
  }

  useEffect(() => {
    if (!supabase || !configured) {
      setLoading(false);
      return;
    }

    let mounted = true;
    supabase.auth.getSession().then(async ({ data }) => {
      if (!mounted) return;
      setSession(data.session);
      if (data.session) {
        await bootstrapProfile(data.session).catch(() => undefined);
      }
      if (mounted) setLoading(false);
    });

    const { data } = supabase.auth.onAuthStateChange((_event, nextSession) => {
      setSession(nextSession);
      if (nextSession) void bootstrapProfile(nextSession).catch(() => undefined);
      else setProfile(null);
    });

    return () => {
      mounted = false;
      data.subscription.unsubscribe();
    };
  }, [supabase, configured]);

  const value: AuthContextValue = {
    configured,
    apiOnline,
    loading,
    user: session?.user ?? null,
    profile,
    accessToken: session?.access_token ?? null,
    async signIn(email, password) {
      if (!supabase) throw new Error("Online accounts are not configured for this demo.");
      const { error } = await supabase.auth.signInWithPassword({ email, password });
      if (error) throw error;
    },
    async signUp(email, password) {
      if (!supabase) throw new Error("Online accounts are not configured for this demo.");
      const { error } = await supabase.auth.signUp({ email, password });
      if (error) throw error;
    },
    async signOut() {
      if (supabase) await supabase.auth.signOut();
      setSession(null);
      setProfile(null);
    },
    bootstrapProfile,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth() {
  const value = useContext(AuthContext);
  if (!value) throw new Error("useAuth must be used inside AuthProvider");
  return value;
}
