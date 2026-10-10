import { gatewayConfig, handleGateway } from "@/lib/server/session-gateway";

export const runtime = "nodejs";
export const dynamic = "force-dynamic";
export const maxDuration = 300;

async function handle(request: Request): Promise<Response> {
  try {
    return await handleGateway(request, gatewayConfig(process.env));
  } catch {
    return Response.json({ code: "SERVICE_UNAVAILABLE", message: "The gateway is not configured." }, {
      status: 503, headers: { "Cache-Control": "no-store" },
    });
  }
}

export { handle as GET, handle as POST, handle as PATCH, handle as DELETE, handle as PUT };
