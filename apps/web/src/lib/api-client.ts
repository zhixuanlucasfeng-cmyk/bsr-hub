import { getAuthConfig } from "./auth-config";

export class ApiClientError extends Error {
  constructor(
    message: string,
    public readonly status: number,
    public readonly code: string,
  ) {
    super(message);
  }
}

export async function apiFetch<T>(
  path: string,
  accessToken: string,
  init: RequestInit = {},
): Promise<T> {
  const config = getAuthConfig();
  if (!config.apiBaseUrl) {
    throw new ApiClientError(
      "The online backend is not connected for this demo.",
      0,
      "API_NOT_CONFIGURED",
    );
  }

  const response = await fetch(`${config.apiBaseUrl}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      authorization: `Bearer ${accessToken}`,
      ...(init.headers ?? {}),
    },
  });

  if (!response.ok) {
    const body = await response.json().catch(() => ({}));
    const error = body.error ?? body;
    throw new ApiClientError(
      error.message ?? "The BSR Hub backend could not complete this action.",
      response.status,
      error.code ?? "API_ERROR",
    );
  }

  return response.json() as Promise<T>;
}
