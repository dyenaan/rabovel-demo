import type { NextConfig } from "next";
import { browserHeaders } from "./src/lib/server/browser-security";

const nextConfig: NextConfig = {
  poweredByHeader: false,
  async headers() {
    return [{ source: "/:path*", headers: browserHeaders(process.env.NODE_ENV === "production") }];
  },
};

export default nextConfig;
