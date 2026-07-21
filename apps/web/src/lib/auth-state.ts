export type AuthGateState = "signed-in" | "signed-out" | "api-unavailable";

export function describeAuthGate(input: {
  authConfigured: boolean;
  signedIn: boolean;
}): AuthGateState {
  if (!input.authConfigured) return "api-unavailable";
  return input.signedIn ? "signed-in" : "signed-out";
}
