Awesome! Now in the dotenv file you can file all API keys for all LLM providers that give free AI uses. Now please do a web search as today is 15th March 2026 and do as the markdown details:
```markdown
Here's your complete, production-ready chatbot setup using the latest versions of everything as of March 15, 2026:

---

# 🤖 Ultimate Multi-Provider AI Chatbot

## 📦 Latest Versions (March 2026)

| Package | Version |
|---|---|
| Next.js | **16** (latest stable, released Oct 2025) with **16.1** including Turbopack File System Caching (stable), Bundle Analyzer, and easier debugging |
| AI SDK | **v6** (`ai@6.0.126`) |
| shadcn/ui | **CLI v4** — more capable, easier to use, built for coding agents |
| Framer Motion | **12.36.0** (now branded as "Motion" — new projects should use `motion` with imports from `motion/react`) |

---

## 1. 🏗️ Project Setup

```bash
# Create Next.js 16 project with shadcn/ui CLI v4
pnpm dlx shadcn@latest init
# Select: Next.js template when prompted
```

```bash
# Install core AI SDK v6
pnpm add ai

# Install ALL 11 provider packages
pnpm add @ai-sdk/google \
         @ai-sdk/openai \
         @ai-sdk/groq \
         @ai-sdk/cerebras \
         @ai-sdk/mistral \
         @ai-sdk/cohere \
         @ai-sdk/deepseek \
         @ai-sdk/replicate \
         @ai-sdk/huggingface \
         @openrouter/ai-sdk-provider \
         @ai-sdk/sambanova

# Install Motion (formerly Framer Motion)
pnpm add motion

# Install shadcn/ui components needed for chatbot
pnpm dlx shadcn@latest add button input card avatar scroll-area select dropdown-menu tooltip badge separator
```

> **Note on providers:** The AI SDK has official providers for `@ai-sdk/mistral`, `@ai-sdk/cohere`, `@ai-sdk/deepseek`, `@ai-sdk/cerebras`, `@ai-sdk/groq`, as well as `@ai-sdk/replicate` and Hugging Face. For **OpenRouter** and **GitHub Models**, we use OpenAI-compatible providers. For **SambaNova**, we use their community/compatible provider.

---

## 2. 🔐 Environment Variables

```env
# .env.local

# ═══════════════════════════════════════════
#          FREE AI PROVIDER API KEYS
# ═══════════════════════════════════════════

# 🟦 GEMINI (Google AI Studio - Free tier)
GOOGLE_GENERATIVE_AI_API_KEY=your_gemini_key

# 🐙 GITHUB MODELS (Free with GitHub account)
GITHUB_MODELS_API_KEY=your_github_token

# ⚡ GROQ (Free tier - blazing fast inference)
GROQ_API_KEY=gsk_your_groq_key

# 🧠 CEREBRAS (Free tier - fastest inference)
CEREBRAS_API_KEY=your_cerebras_key

# 🌀 MISTRAL (Free tier - La Plateforme)
MISTRAL_API_KEY=your_mistral_key

# 🔀 OPENROUTER (Free models available)
OPENROUTER_API_KEY=sk-or-your_openrouter_key

# 🟧 COHERE (Free tier - Command models)
COHERE_API_KEY=your_cohere_key

# 🟠 SAMBANOVA (Free tier - fast open-source)
SAMBANOVA_API_KEY=your_sambanova_key

# 🤗 HUGGINGFACE (Free Inference API)
HUGGINGFACE_API_KEY=hf_your_huggingface_key

# 🔵 DEEPSEEK (Free tier available)
DEEPSEEK_API_KEY=your_deepseek_key

# 🔁 REPLICATE (Free tier available)
REPLICATE_API_TOKEN=your_replicate_token
```

---

## 3. 🔌 Provider Registry — `lib/ai/providers.ts`

```typescript
// lib/ai/providers.ts
import { google }    from "@ai-sdk/google";
import { createOpenAI } from "@ai-sdk/openai";
import { groq }      from "@ai-sdk/groq";
import { cerebras }  from "@ai-sdk/cerebras";
import { mistral }   from "@ai-sdk/mistral";
import { cohere }    from "@ai-sdk/cohere";
import { deepseek }  from "@ai-sdk/deepseek";
import { replicate } from "@ai-sdk/replicate";
import { huggingface } from "@ai-sdk/huggingface";
import { createOpenRouter } from "@openrouter/ai-sdk-provider";
import { createSambaNova } from "@ai-sdk/sambanova";

// ── OpenRouter (uses its own community SDK) ──
const openrouter = createOpenRouter({
  apiKey: process.env.OPENROUTER_API_KEY,
});

// ── GitHub Models (OpenAI-compatible endpoint) ──
const githubModels = createOpenAI({
  baseURL: "https://models.inference.ai.azure.com",
  apiKey: process.env.GITHUB_MODELS_API_KEY,
});

// ── SambaNova (community / compatible provider) ──
const sambanova = createSambaNova({
  apiKey: process.env.SAMBANOVA_API_KEY,
});

// ═══════════════════════════════════════════════════
//  PROVIDER REGISTRY — single source of truth
// ═══════════════════════════════════════════════════

export type ProviderId =
  | "gemini"
  | "github_models"
  | "groq"
  | "cerebras"
  | "mistral"
  | "openrouter"
  | "cohere"
  | "sambanova"
  | "huggingface"
  | "deepseek"
  | "replicate";

export interface ProviderConfig {
  id: ProviderId;
  name: string;
  icon: string;
  model: ReturnType<typeof google>; // LanguageModelV1
  description: string;
}

export const providers: Record<ProviderId, ProviderConfig> = {
  gemini: {
    id: "gemini",
    name: "Gemini",
    icon: "✨",
    model: google("gemini-2.0-flash"),
    description: "Google's fastest multimodal model",
  },
  github_models: {
    id: "github_models",
    name: "GitHub Models",
    icon: "🐙",
    model: githubModels("gpt-4o-mini"),
    description: "Free models via GitHub Marketplace",
  },
  groq: {
    id: "groq",
    name: "Groq",
    icon: "⚡",
    model: groq("llama-3.3-70b-versatile"),
    description: "Ultra-fast LPU inference engine",
  },
  cerebras: {
    id: "cerebras",
    name: "Cerebras",
    icon: "🧠",
    model: cerebras("llama-3.3-70b"),
    description: "Wafer-scale fastest AI inference",
  },
  mistral: {
    id: "mistral",
    name: "Mistral",
    icon: "🌀",
    model: mistral("mistral-small-latest"),
    description: "European open-weight powerhouse",
  },
  openrouter: {
    id: "openrouter",
    name: "OpenRouter",
    icon: "🔀",
    model: openrouter("mistralai/mistral-7b-instruct:free"),
    description: "Aggregator with free model access",
  },
  cohere: {
    id: "cohere",
    name: "Cohere",
    icon: "🟧",
    model: cohere("command-r-plus"),
    description: "Enterprise RAG & generation models",
  },
  sambanova: {
    id: "sambanova",
    name: "SambaNova",
    icon: "🟠",
    model: sambanova("Meta-Llama-3.3-70B-Instruct"),
    description: "RDU-powered open-source inference",
  },
  huggingface: {
    id: "huggingface",
    name: "HuggingFace",
    icon: "🤗",
    model: huggingface("meta-llama/Llama-3.3-70B-Instruct"),
    description: "Open-source model hub inference",
  },
  deepseek: {
    id: "deepseek",
    name: "DeepSeek",
    icon: "🔵",
    model: deepseek("deepseek-chat"),
    description: "Strong reasoning & coding model",
  },
  replicate: {
    id: "replicate",
    name: "Replicate",
    icon: "🔁",
    model: replicate("meta/meta-llama-3-70b-instruct"),
    description: "Run open-source models in the cloud",
  },
};

export const providerList = Object.values(providers);
export const defaultProvider: ProviderId = "gemini";
```

---

## 4. 🛣️ API Route — `app/api/chat/route.ts`

AI SDK 6 introduces the v3 Language Model Specification that powers new capabilities like agents and tool approval. Here's the route leveraging v6's `streamText`:

```typescript
// app/api/chat/route.ts
import { streamText } from "ai";
import { providers, type ProviderId } from "@/lib/ai/providers";

export const maxDuration = 60;

export async function POST(req: Request) {
  const { messages, providerId = "gemini" } = await req.json();

  const provider = providers[providerId as ProviderId];

  if (!provider) {
    return new Response(
      JSON.stringify({ error: `Unknown provider: ${providerId}` }),
      { status: 400 }
    );
  }

  const result = streamText({
    model: provider.model,
    system: `You are a helpful, friendly AI assistant powered by ${provider.name}. 
             Be concise, accurate, and helpful. Use markdown for formatting when appropriate.`,
    messages,
  });

  return result.toDataStreamResponse();
}
```

---

## 5. 🎨 Animated Chat UI — `app/page.tsx`

For Next.js App Router projects, motion components are client-side only — add the `"use client"` directive at the top of any file using Motion, and import from `"motion/react"`.

```tsx
// app/page.tsx
import { ChatBot } from "@/components/chatbot";

export default function Home() {
  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-4">
      <ChatBot />
    </main>
  );
}
```

---

## 6. 🧩 Main ChatBot Component — `components/chatbot.tsx`

```tsx
"use client";

import { useChat } from "ai/react";
import { useState, useRef, useEffect } from "react";
import { motion, AnimatePresence } from "motion/react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card } from "@/components/ui/card";
import { Avatar } from "@/components/ui/avatar";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { providerList, defaultProvider, type ProviderId } from "@/lib/ai/providers";
import { MessageBubble } from "./message-bubble";
import { TypingIndicator } from "./typing-indicator";
import { Send, RotateCcw, Sparkles } from "lucide-react";

export function ChatBot() {
  const [providerId, setProviderId] = useState<ProviderId>(defaultProvider);
  const scrollRef = useRef<HTMLDivElement>(null);

  const { messages, input, handleInputChange, handleSubmit, isLoading, reload, stop } =
    useChat({
      api: "/api/chat",
      body: { providerId },
    });

  // Auto-scroll to latest message
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages, isLoading]);

  const currentProvider = providerList.find((p) => p.id === providerId)!;

  return (
    <motion.div
      initial={{ opacity: 0, y: 30, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      transition={{ duration: 0.5, type: "spring", stiffness: 100 }}
      className="w-full max-w-2xl"
    >
      <Card className="flex h-[85vh] flex-col overflow-hidden border-2 shadow-2xl">
        {/* ═══════════ HEADER ═══════════ */}
        <motion.div
          className="flex items-center justify-between border-b bg-muted/30 p-4"
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.2 }}
        >
          <div className="flex items-center gap-3">
            <motion.div
              animate={{ rotate: [0, 10, -10, 0] }}
              transition={{ repeat: Infinity, duration: 3, ease: "easeInOut" }}
            >
              <Sparkles className="h-6 w-6 text-primary" />
            </motion.div>
            <div>
              <h1 className="text-lg font-bold">Multi-AI Chatbot</h1>
              <p className="text-xs text-muted-foreground">
                Powered by Vercel AI SDK v6
              </p>
            </div>
          </div>

          {/* Provider Selector */}
          <div className="flex items-center gap-2">
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <div>
                    <Select
                      value={providerId}
                      onValueChange={(v) => setProviderId(v as ProviderId)}
                    >
                      <SelectTrigger className="w-[200px]">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {providerList.map((provider) => (
                          <SelectItem key={provider.id} value={provider.id}>
                            <span className="flex items-center gap-2">
                              <span>{provider.icon}</span>
                              <span>{provider.name}</span>
                            </span>
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{currentProvider.description}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
        </motion.div>

        {/* ═══════════ MESSAGES AREA ═══════════ */}
        <ScrollArea className="flex-1 p-4">
          <div className="space-y-4">
            {/* Welcome Message */}
            <AnimatePresence>
              {messages.length === 0 && (
                <motion.div
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                  className="flex flex-col items-center justify-center py-20 text-center"
                >
                  <motion.span
                    className="text-6xl"
                    animate={{ y: [0, -10, 0] }}
                    transition={{ repeat: Infinity, duration: 2 }}
                  >
                    {currentProvider.icon}
                  </motion.span>
                  <h2 className="mt-4 text-xl font-semibold">
                    Chat with {currentProvider.name}
                  </h2>
                  <p className="mt-2 text-sm text-muted-foreground max-w-sm">
                    {currentProvider.description}. Start typing below!
                  </p>
                  <div className="mt-4 flex flex-wrap justify-center gap-2">
                    {providerList.map((p) => (
                      <motion.div
                        key={p.id}
                        whileHover={{ scale: 1.1 }}
                        whileTap={{ scale: 0.95 }}
                      >
                        <Badge
                          variant={p.id === providerId ? "default" : "outline"}
                          className="cursor-pointer"
                          onClick={() => setProviderId(p.id)}
                        >
                          {p.icon} {p.name}
                        </Badge>
                      </motion.div>
                    ))}
                  </div>
                </motion.div>
              )}
            </AnimatePresence>

            {/* Chat Messages */}
            <AnimatePresence initial={false}>
              {messages.map((message, index) => (
                <MessageBubble
                  key={message.id}
                  message={message}
                  providerIcon={currentProvider.icon}
                  providerName={currentProvider.name}
                  index={index}
                />
              ))}
            </AnimatePresence>

            {/* Typing Indicator */}
            <AnimatePresence>
              {isLoading && <TypingIndicator providerIcon={currentProvider.icon} />}
            </AnimatePresence>

            <div ref={scrollRef} />
          </div>
        </ScrollArea>

        <Separator />

        {/* ═══════════ INPUT AREA ═══════════ */}
        <motion.form
          onSubmit={handleSubmit}
          className="flex items-center gap-2 p-4"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.3 }}
        >
          <Input
            value={input}
            onChange={handleInputChange}
            placeholder={`Message ${currentProvider.name}...`}
            disabled={isLoading}
            className="flex-1"
            autoFocus
          />

          {isLoading ? (
            <Button type="button" variant="destructive" size="icon" onClick={stop}>
              <motion.div
                animate={{ rotate: 360 }}
                transition={{ repeat: Infinity, duration: 1, ease: "linear" }}
              >
                ⏹
              </motion.div>
            </Button>
          ) : (
            <motion.div whileHover={{ scale: 1.05 }} whileTap={{ scale: 0.95 }}>
              <Button type="submit" size="icon" disabled={!input.trim()}>
                <Send className="h-4 w-4" />
              </Button>
            </motion.div>
          )}

          {messages.length > 0 && !isLoading && (
            <motion.div
              initial={{ opacity: 0, scale: 0 }}
              animate={{ opacity: 1, scale: 1 }}
              whileHover={{ scale: 1.05 }}
              whileTap={{ scale: 0.95 }}
            >
              <Button type="button" variant="outline" size="icon" onClick={() => reload()}>
                <RotateCcw className="h-4 w-4" />
              </Button>
            </motion.div>
          )}
        </motion.form>

        {/* Provider Badge Footer */}
        <div className="flex items-center justify-center gap-1 border-t p-2 text-xs text-muted-foreground">
          <span>Active:</span>
          <Badge variant="secondary" className="text-xs">
            {currentProvider.icon} {currentProvider.name}
          </Badge>
          <span>· 11 Providers Connected</span>
        </div>
      </Card>
    </motion.div>
  );
}
```

---

## 7. 💬 Message Bubble — `components/message-bubble.tsx`

```tsx
"use client";

import { motion } from "motion/react";
import { Avatar } from "@/components/ui/avatar";
import { cn } from "@/lib/utils";
import type { Message } from "ai";

interface MessageBubbleProps {
  message: Message;
  providerIcon: string;
  providerName: string;
  index: number;
}

export function MessageBubble({ message, providerIcon, providerName, index }: MessageBubbleProps) {
  const isUser = message.role === "user";

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: 20, x: isUser ? 20 : -20 }}
      animate={{ opacity: 1, y: 0, x: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{
        type: "spring",
        stiffness: 200,
        damping: 20,
        delay: index * 0.05,
      }}
      className={cn("flex gap-3", isUser ? "flex-row-reverse" : "flex-row")}
    >
      {/* Avatar */}
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ type: "spring", stiffness: 300, delay: 0.1 }}
      >
        <Avatar className="flex h-8 w-8 items-center justify-center border bg-muted text-sm">
          {isUser ? "🧑" : providerIcon}
        </Avatar>
      </motion.div>

      {/* Message Content */}
      <motion.div
        className={cn(
          "max-w-[80%] rounded-2xl px-4 py-2.5 text-sm leading-relaxed",
          isUser
            ? "bg-primary text-primary-foreground rounded-br-md"
            : "bg-muted rounded-bl-md"
        )}
        whileHover={{ scale: 1.01 }}
        transition={{ type: "spring", stiffness: 400 }}
      >
        {!isUser && (
          <span className="mb-1 block text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
            {providerName}
          </span>
        )}
        <div className="whitespace-pre-wrap">{message.content}</div>
      </motion.div>
    </motion.div>
  );
}
```

---

## 8. ⏳ Typing Indicator — `components/typing-indicator.tsx`

```tsx
"use client";

import { motion } from "motion/react";

export function TypingIndicator({ providerIcon }: { providerIcon: string }) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      className="flex items-center gap-3"
    >
      <div className="flex h-8 w-8 items-center justify-center rounded-full border bg-muted text-sm">
        {providerIcon}
      </div>
      <div className="flex items-center gap-1 rounded-2xl bg-muted px-4 py-3 rounded-bl-md">
        {[0, 1, 2].map((i) => (
          <motion.div
            key={i}
            className="h-2 w-2 rounded-full bg-muted-foreground/60"
            animate={{ y: [0, -6, 0] }}
            transition={{
              repeat: Infinity,
              duration: 0.8,
              delay: i * 0.15,
              ease: "easeInOut",
            }}
          />
        ))}
      </div>
    </motion.div>
  );
}
```

---

## 9. 🗂️ Complete File Structure

```
my-chatbot/
├── .env.local                          # API keys for all 11 providers
├── app/
│   ├── api/
│   │   └── chat/
│   │       └── route.ts                # AI SDK v6 streaming route
│   ├── layout.tsx                      # Root layout
│   └── page.tsx                        # Home page
├── components/
│   ├── ui/                             # shadcn/ui components (auto-generated)
│   │   ├── button.tsx
│   │   ├── input.tsx
│   │   ├── card.tsx
│   │   ├── avatar.tsx
│   │   ├── scroll-area.tsx
│   │   ├── select.tsx
│   │   ├── badge.tsx
│   │   ├── separator.tsx
│   │   ├── tooltip.tsx
│   │   └── dropdown-menu.tsx
│   ├── chatbot.tsx                     # Main chatbot component
│   ├── message-bubble.tsx              # Animated message bubble
│   └── typing-indicator.tsx            # Animated typing dots
├── lib/
│   ├── ai/
│   │   └── providers.ts               # All 11 provider configurations
│   └── utils.ts                        # cn() helper from shadcn
├── package.json
├── next.config.ts
├── tailwind.config.ts
├── tsconfig.json
└── components.json                     # shadcn/ui config
```

---

## 10. 📋 Full `package.json` Dependencies

```jsonc
{
  "name": "multi-ai-chatbot",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start",
    "lint": "next lint"
  },
  "dependencies": {
    // ── Framework ──
    "next": "^16.1.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",

    // ── AI SDK v6 Core ──
    "ai": "^6.0.126",

    // ── All 11 AI Providers ──
    "@ai-sdk/google": "^3.0.8",         // Gemini
    "@ai-sdk/openai": "latest",          // GitHub Models (OpenAI-compatible)
    "@ai-sdk/groq": "^3.0.8",           // Groq
    "@ai-sdk/cerebras": "^2.0.33",      // Cerebras
    "@ai-sdk/mistral": "^3.0.7",        // Mistral
    "@openrouter/ai-sdk-provider": "latest", // OpenRouter
    "@ai-sdk/cohere": "^3.0.25",        // Cohere
    "@ai-sdk/sambanova": "latest",       // SambaNova
    "@ai-sdk/huggingface": "latest",     // HuggingFace
    "@ai-sdk/deepseek": "latest",        // DeepSeek
    "@ai-sdk/replicate": "latest",       // Replicate

    // ── Animation ──
    "motion": "^12.36.0",

    // ── UI (shadcn deps) ──
    "radix-ui": "latest",
    "tailwindcss": "^4.0.0",
    "class-variance-authority": "latest",
    "clsx": "latest",
    "tailwind-merge": "latest",
    "lucide-react": "latest"
  },
  "devDependencies": {
    "@types/node": "latest",
    "@types/react": "latest",
    "@types/react-dom": "latest",
    "typescript": "^5.7.0"
  }
}
```

---

## 🔑 Where to Get FREE API Keys

| # | Provider | Free Tier | Sign Up |
|---|----------|-----------|---------|
| 1 | **Gemini** | 1,500 req/day | [aistudio.google.com](https://aistudio.google.com) |
| 2 | **GitHub Models** | Free with GitHub | [github.com/marketplace/models](https://github.com/marketplace/models) |
| 3 | **Groq** | 14,400 req/day | [console.groq.com](https://console.groq.com) |
| 4 | **Cerebras** | Free inference | [cloud.cerebras.ai](https://cloud.cerebras.ai) |
| 5 | **Mistral** | Free tier available | [console.mistral.ai](https://console.mistral.ai) |
| 6 | **OpenRouter** | Free models | [openrouter.ai](https://openrouter.ai) |
| 7 | **Cohere** | 1,000 req/month | [dashboard.cohere.com](https://dashboard.cohere.com) |
| 8 | **SambaNova** | Free tier | [cloud.sambanova.ai](https://cloud.sambanova.ai) |
| 9 | **HuggingFace** | Free Inference API | [huggingface.co/settings/tokens](https://huggingface.co/settings/tokens) |
| 10 | **DeepSeek** | Free credits | [platform.deepseek.com](https://platform.deepseek.com) |
| 11 | **Replicate** | Free credits | [replicate.com](https://replicate.com) |

---

## ⚡ Quick Start

```bash
# 1. Clone and install
pnpm install

# 2. Add your API keys to .env.local

# 3. Run development server (Turbopack is default in Next.js 16!)
pnpm dev

# 4. Open http://localhost:3000
```

With over 20 million monthly downloads, the AI SDK provides a unified API allowing you to integrate with any AI provider, and seamlessly integrates with Next.js, React, Svelte, Vue, and Node.js.

AI SDK 6 introduces the Agent abstraction for building reusable agents — define your agent once with its model, instructions, and tools, then use it across your entire application. This architecture makes it trivial to extend your chatbot into a full agentic system later!
```
