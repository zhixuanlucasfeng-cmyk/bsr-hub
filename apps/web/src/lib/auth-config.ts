export type AuthConfig = {
  supabaseUrl: string;
  supabaseAnonKey: string;
  apiBaseUrl: string;
  staticDemo: boolean;
};

export function getAuthConfig(env: NodeJS.ProcessEnv = process.env): AuthConfig {
  return {
    supabaseUrl: env.NEXT_PUBLIC_SUPABASE_URL ?? "",
    supabaseAnonKey: env.NEXT_PUBLIC_SUPABASE_ANON_KEY ?? "",
    apiBaseUrl: (env.NEXT_PUBLIC_API_BASE_URL ?? env.NEXT_PUBLIC_API_URL ?? "").replace(
      /\/$/,
      "",
    ),
    staticDemo: env.NEXT_PUBLIC_STATIC_DEMO === "true",
  };
}

export function isAuthConfigured(config: AuthConfig): boolean {
  return Boolean(
    config.supabaseUrl &&
      config.supabaseAnonKey &&
      config.apiBaseUrl &&
      !config.staticDemo,
  );
}
