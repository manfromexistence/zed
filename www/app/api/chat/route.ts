// app/api/chat/route.ts
import { streamText } from "ai";
import { getModel, type ProviderId } from "@/lib/ai/providers";

export const maxDuration = 60;

export async function POST(req: Request) {
  try {
    const {
      messages,
      providerId = "gemini",
      modelId = "gemini-3.1-flash-lite-preview",
    } = await req.json();

    const model = getModel(providerId as ProviderId, modelId);
    if (!model) {
      return new Response(
        JSON.stringify({
          error: `Unknown provider/model: ${providerId}/${modelId}`,
        }),
        { status: 400 },
      );
    }

    const result = streamText({
      model,
      system: `You are a helpful, friendly AI assistant. Be concise, accurate, and helpful. Use markdown for formatting when appropriate.`,
      messages,
    });

    return result.toUIMessageStreamResponse();
  } catch (error) {
    console.error("Chat API error:", error);
    return new Response(
      JSON.stringify({
        error: error instanceof Error ? error.message : "Unknown error",
      }),
      { status: 500 },
    );
  }
}
