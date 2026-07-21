export type AuthConfig = {
  supabaseUrl: string;
  supabaseAnonKey: string;
  apiBaseUrl: string;
  staticDemo: boolean;
};

// Next.js only inlines NEXT_PUBLIC_* values when they are referenced directly.
// Keep this object explicit so the browser bundle receives the deployment config.
const publicEnv = {
  NEXT_PUBLIC_SUPABASE_URL: process.env.NEXT_PUBLIC_SUPABASE_URL,
  NEXT_PUBLIC_SUPABASE_ANON_KEY: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY,
  NEXT_PUBLIC_API_BASE_URL: process.env.NEXT_PUBLIC_API_BASE_URL,
  NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
  NEXT_PUBLIC_STATIC_DEMO: process.env.NEXT_PUBLIC_STATIC_DEMO,
};

export function getAuthConfig(
  env: Partial<NodeJS.ProcessEnv> = publicEnv,
): AuthConfig {
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
