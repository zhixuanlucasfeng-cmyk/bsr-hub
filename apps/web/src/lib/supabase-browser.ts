"use client";

import { createClient, type SupabaseClient } from "@supabase/supabase-js";
import { getAuthConfig, isAuthConfigured } from "./auth-config";

let client: SupabaseClient | null = null;

export function createBrowserSupabaseClient(): SupabaseClient | null {
  const config = getAuthConfig();
  if (!isAuthConfigured(config)) return null;
  client ??= createClient(config.supabaseUrl, config.supabaseAnonKey);
  return client;
}
