/** Simulates network latency in mock mode so loading states are visible during development. */
export function mockDelay(ms = 400): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
