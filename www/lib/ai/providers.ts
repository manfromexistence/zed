// lib/ai/providers.ts

import type React from "react";
import { bedrock } from "@ai-sdk/amazon-bedrock";
import { anthropic } from "@ai-sdk/anthropic";
import { createAzure } from "@ai-sdk/azure";
import { cerebras } from "@ai-sdk/cerebras";
import { cohere } from "@ai-sdk/cohere";
import { deepseek } from "@ai-sdk/deepseek";
import { google } from "@ai-sdk/google";
import { groq } from "@ai-sdk/groq";
import { mistral } from "@ai-sdk/mistral";
import { createOpenAI } from "@ai-sdk/openai";
import { createOpenAICompatible } from "@ai-sdk/openai-compatible";
import { xai } from "@ai-sdk/xai";
import { createOpenRouter } from "@openrouter/ai-sdk-provider";
import { 
  AnthropicLogo,
  CohereLogoIcon,
  GoogleLogo,
  MetaLogo,
  MistralLogo,
  OpenAILogo,
} from "@/components/chat/provider-logos";

// ── OpenRouter (uses its own community SDK) ──
const openrouter = createOpenRouter({
  apiKey: process.env.OPENROUTER_API_KEY,
});

// ── GitHub Models (OpenAI-compatible endpoint) ──
const githubModels = createOpenAI({
  baseURL: "https://models.inference.ai.azure.com",
  apiKey: process.env.GITHUB_MODELS_API_KEY,
});

// ── Azure OpenAI ──
const azure = createAzure({
  apiKey: process.env.AZURE_API_KEY,
  resourceName: process.env.AZURE_RESOURCE_NAME,
});

// ── Fireworks AI (OpenAI-compatible) ──
const fireworks = createOpenAICompatible({
  name: "fireworks",
  baseURL: "https://api.fireworks.ai/inference/v1",
  apiKey: process.env.FIREWORKS_API_KEY,
});

// ── Together AI (OpenAI-compatible) ──
const together = createOpenAICompatible({
  name: "together",
  baseURL: "https://api.together.xyz/v1",
  apiKey: process.env.TOGETHER_API_KEY,
});

// ── Perplexity (OpenAI-compatible) ──
const perplexity = createOpenAICompatible({
  name: "perplexity",
  baseURL: "https://api.perplexity.ai",
  apiKey: process.env.PERPLEXITY_API_KEY,
});

// ── Ollama (local, OpenAI-compatible) ──
const ollama = createOpenAICompatible({
  name: "ollama",
  baseURL: "http://localhost:11434/v1",
  apiKey: "ollama", // Ollama doesn't require a real API key
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
  | "deepseek"
  | "anthropic"
  | "xai"
  | "azure"
  | "bedrock"
  | "fireworks"
  | "together"
  | "perplexity"
  | "ollama"
  | "ai-gateway"; // AI Gateway for 20+ providers

export interface ModelConfig {
  id: string;
  name: string;
  modelId: string;
  description: string;
}

export interface ProviderConfig {
  id: ProviderId;
  name: string;
  icon: React.ComponentType<{ className?: string }>;
  models: ModelConfig[];
  defaultModel: string;
  description: string;
}

export const providers: Record<ProviderId, ProviderConfig> = {
  "ai-gateway": {
    id: "ai-gateway",
    name: "AI Gateway",
    icon: OpenAILogo,
    models: [
      {
        id: "openai-gpt-4o",
        name: "GPT-4o",
        modelId: "openai/gpt-4o",
        description: "OpenAI's most capable model",
      },
      {
        id: "anthropic-claude-opus-4",
        name: "Claude Opus 4",
        modelId: "anthropic/claude-opus-4",
        description: "Anthropic's most powerful model",
      },
      {
        id: "google-gemini-3-flash",
        name: "Gemini 3 Flash",
        modelId: "google/gemini-3-flash",
        description: "Google's fastest model",
      },
      {
        id: "meta-llama-4",
        name: "Llama 4",
        modelId: "meta/llama-4",
        description: "Meta's latest open model",
      },
      {
        id: "xai-grok-3",
        name: "Grok 3",
        modelId: "xai/grok-3",
        description: "xAI's latest model",
      },
    ],
    defaultModel: "openai-gpt-4o",
    description: "Access 20+ providers through AI Gateway",
  },
  gemini: {
    id: "gemini",
    name: "Google",
    icon: GoogleLogo,
    models: [
      {
        id: "gemini-3.1-flash-lite-preview",
        name: "Gemini 3.1 Flash-Lite",
        modelId: "gemini-3.1-flash-lite-preview",
        description: "Fastest and most cost-effective (March 2026)",
      },
      {
        id: "gemini-2.5-flash",
        name: "Gemini 2.5 Flash",
        modelId: "gemini-2.5-flash",
        description: "Best price-performance for reasoning",
      },
      {
        id: "gemini-2.0-flash-exp",
        name: "Gemini 2.0 Flash Experimental",
        modelId: "gemini-2.0-flash-exp",
        description: "Experimental multimodal model",
      },
    ],
    defaultModel: "gemini-3.1-flash-lite-preview",
    description: "Google's latest multimodal AI models",
  },
  anthropic: {
    id: "anthropic",
    name: "Anthropic",
    icon: AnthropicLogo,
    models: [
      {
        id: "claude-opus-4",
        name: "Claude Opus 4",
        modelId: "claude-opus-4",
        description: "Most capable Claude model",
      },
      {
        id: "claude-sonnet-4",
        name: "Claude Sonnet 4",
        modelId: "claude-sonnet-4",
        description: "Balanced performance and speed",
      },
      {
        id: "claude-haiku-4",
        name: "Claude Haiku 4",
        modelId: "claude-haiku-4",
        description: "Fastest Claude model",
      },
    ],
    defaultModel: "claude-sonnet-4",
    description: "Anthropic's Claude models with extended thinking",
  },
  xai: {
    id: "xai",
    name: "xAI",
    icon: OpenAILogo,
    models: [
      {
        id: "grok-3",
        name: "Grok 3",
        modelId: "grok-3",
        description: "Latest Grok model",
      },
      {
        id: "grok-2",
        name: "Grok 2",
        modelId: "grok-2",
        description: "Previous generation Grok",
      },
    ],
    defaultModel: "grok-3",
    description: "xAI's Grok models with real-time data",
  },
  github_models: {
    id: "github_models",
    name: "GitHub",
    icon: OpenAILogo,
    models: [
      {
        id: "gpt-4o-mini",
        name: "GPT-4o Mini",
        modelId: "gpt-4o-mini",
        description: "Fast and efficient model",
      },
      {
        id: "gpt-4o",
        name: "GPT-4o",
        modelId: "gpt-4o",
        description: "Most capable OpenAI model",
      },
    ],
    defaultModel: "gpt-4o-mini",
    description: "Latest OpenAI models via GitHub",
  },
  groq: {
    id: "groq",
    name: "Groq",
    icon: MetaLogo,
    models: [
      {
        id: "llama-3.1-8b-instant",
        name: "Llama 3.1 8B",
        modelId: "llama-3.1-8b-instant",
        description: "Fast 8B parameter model",
      },
      {
        id: "llama-3.3-70b-versatile",
        name: "Llama 3.3 70B",
        modelId: "llama-3.3-70b-versatile",
        description: "Versatile 70B parameter model",
      },
      {
        id: "mixtral-8x7b-32768",
        name: "Mixtral 8x7B",
        modelId: "mixtral-8x7b-32768",
        description: "Mixture of experts model",
      },
    ],
    defaultModel: "llama-3.1-8b-instant",
    description: "Ultra-fast LPU inference engine",
  },
  cerebras: {
    id: "cerebras",
    name: "Cerebras",
    icon: MetaLogo,
    models: [
      {
        id: "llama3.1-8b",
        name: "Llama 3.1 8B",
        modelId: "llama3.1-8b",
        description: "8B parameter model",
      },
      {
        id: "llama3.1-70b",
        name: "Llama 3.1 70B",
        modelId: "llama3.1-70b",
        description: "70B parameter model",
      },
    ],
    defaultModel: "llama3.1-8b",
    description: "Wafer-scale fastest AI inference",
  },
  mistral: {
    id: "mistral",
    name: "Mistral",
    icon: MistralLogo,
    models: [
      {
        id: "mistral-small-latest",
        name: "Mistral Small",
        modelId: "mistral-small-latest",
        description: "Cost-effective model",
      },
      {
        id: "mistral-large-latest",
        name: "Mistral Large",
        modelId: "mistral-large-latest",
        description: "Most capable Mistral model",
      },
    ],
    defaultModel: "mistral-small-latest",
    description: "European open-weight powerhouse",
  },
  openrouter: {
    id: "openrouter",
    name: "OpenRouter",
    icon: OpenAILogo,
    models: [
      {
        id: "deepseek-r1-free",
        name: "DeepSeek R1",
        modelId: "deepseek/deepseek-r1:free",
        description: "Latest reasoning model (free)",
      },
      {
        id: "gemini-3.1-flash-lite-free",
        name: "Gemini 3.1 Flash-Lite",
        modelId: "google/gemini-3.1-flash-lite:free",
        description: "Google's fastest model (free)",
      },
      {
        id: "llama-4-scout-free",
        name: "Llama 4 Scout",
        modelId: "meta-llama/llama-4-scout:free",
        description: "Latest Llama model (free)",
      },
      {
        id: "claude-opus-4",
        name: "Claude Opus 4",
        modelId: "anthropic/claude-opus-4",
        description: "Most capable Claude (paid)",
      },
      {
        id: "gpt-4o",
        name: "GPT-4o",
        modelId: "openai/gpt-4o",
        description: "OpenAI's best model (paid)",
      },
    ],
    defaultModel: "deepseek-r1-free",
    description: "300+ models aggregator (free & paid)",
  },
  cohere: {
    id: "cohere",
    name: "Cohere",
    icon: CohereLogoIcon,
    models: [
      {
        id: "command-r-plus",
        name: "Command R+",
        modelId: "command-r-plus",
        description: "Enhanced capabilities",
      },
      {
        id: "command-r",
        name: "Command R",
        modelId: "command-r",
        description: "Standard model",
      },
    ],
    defaultModel: "command-r-plus",
    description: "Enterprise RAG & generation models",
  },
  deepseek: {
    id: "deepseek",
    name: "DeepSeek",
    icon: AnthropicLogo,
    models: [
      {
        id: "deepseek-v3.2",
        name: "DeepSeek V3.2",
        modelId: "deepseek-v3.2",
        description: "Latest chat model (March 2026)",
      },
      {
        id: "deepseek-r1",
        name: "DeepSeek R1",
        modelId: "deepseek-r1",
        description: "Reasoning model",
      },
    ],
    defaultModel: "deepseek-v3.2",
    description: "Latest reasoning & coding models",
  },
  azure: {
    id: "azure",
    name: "Azure OpenAI",
    icon: OpenAILogo,
    models: [
      {
        id: "gpt-4o",
        name: "GPT-4o",
        modelId: "gpt-4o",
        description: "OpenAI via Azure",
      },
      {
        id: "gpt-4o-mini",
        name: "GPT-4o Mini",
        modelId: "gpt-4o-mini",
        description: "Fast model via Azure",
      },
    ],
    defaultModel: "gpt-4o",
    description: "OpenAI models via Azure",
  },
  bedrock: {
    id: "bedrock",
    name: "AWS Bedrock",
    icon: OpenAILogo,
    models: [
      {
        id: "claude-opus-4",
        name: "Claude Opus 4",
        modelId: "anthropic.claude-opus-4",
        description: "Claude via AWS",
      },
      {
        id: "llama-3-70b",
        name: "Llama 3 70B",
        modelId: "meta.llama3-70b-instruct-v1:0",
        description: "Llama via AWS",
      },
    ],
    defaultModel: "claude-opus-4",
    description: "Multiple models via AWS Bedrock",
  },
  fireworks: {
    id: "fireworks",
    name: "Fireworks AI",
    icon: MetaLogo,
    models: [
      {
        id: "llama-3.1-70b",
        name: "Llama 3.1 70B",
        modelId: "accounts/fireworks/models/llama-v3p1-70b-instruct",
        description: "Fast Llama inference",
      },
      {
        id: "mixtral-8x7b",
        name: "Mixtral 8x7B",
        modelId: "accounts/fireworks/models/mixtral-8x7b-instruct",
        description: "Mixture of experts",
      },
    ],
    defaultModel: "llama-3.1-70b",
    description: "Fast inference for open models",
  },
  together: {
    id: "together",
    name: "Together AI",
    icon: MetaLogo,
    models: [
      {
        id: "llama-3.1-70b",
        name: "Llama 3.1 70B",
        modelId: "meta-llama/Meta-Llama-3.1-70B-Instruct-Turbo",
        description: "Fast Llama inference",
      },
      {
        id: "qwen-2.5-72b",
        name: "Qwen 2.5 72B",
        modelId: "Qwen/Qwen2.5-72B-Instruct-Turbo",
        description: "Alibaba's model",
      },
    ],
    defaultModel: "llama-3.1-70b",
    description: "Fast inference platform",
  },
  perplexity: {
    id: "perplexity",
    name: "Perplexity",
    icon: OpenAILogo,
    models: [
      {
        id: "llama-3.1-sonar-large",
        name: "Sonar Large",
        modelId: "llama-3.1-sonar-large-128k-online",
        description: "Online search-enabled",
      },
      {
        id: "llama-3.1-sonar-small",
        name: "Sonar Small",
        modelId: "llama-3.1-sonar-small-128k-online",
        description: "Fast online search",
      },
    ],
    defaultModel: "llama-3.1-sonar-large",
    description: "Search-augmented models",
  },
  ollama: {
    id: "ollama",
    name: "Ollama (Local)",
    icon: MetaLogo,
    models: [
      {
        id: "llama3.1",
        name: "Llama 3.1",
        modelId: "llama3.1",
        description: "Local Llama 3.1",
      },
      {
        id: "mistral",
        name: "Mistral",
        modelId: "mistral",
        description: "Local Mistral",
      },
      {
        id: "codellama",
        name: "Code Llama",
        modelId: "codellama",
        description: "Local coding model",
      },
    ],
    defaultModel: "llama3.1",
    description: "Run models locally with Ollama",
  },
};

// Helper function to get model by provider and model ID
export function getModel(providerId: ProviderId, modelId: string) {
  const provider = providers[providerId];
  if (!provider) return null;

  const modelConfig = provider.models.find((m) => m.id === modelId);
  if (!modelConfig) return null;

  // Create the actual model instance based on provider
  switch (providerId) {
    case "ai-gateway":
      // AI Gateway uses string format: "provider/model"
      return modelConfig.modelId;
    case "gemini":
      return google(modelConfig.modelId);
    case "anthropic":
      return anthropic(modelConfig.modelId);
    case "xai":
      return xai(modelConfig.modelId);
    case "github_models":
      return githubModels(modelConfig.modelId);
    case "groq":
      return groq(modelConfig.modelId);
    case "cerebras":
      return cerebras(modelConfig.modelId);
    case "mistral":
      return mistral(modelConfig.modelId);
    case "openrouter":
      return openrouter(modelConfig.modelId);
    case "cohere":
      return cohere(modelConfig.modelId);
    case "deepseek":
      return deepseek(modelConfig.modelId);
    case "azure":
      return azure(modelConfig.modelId);
    case "bedrock":
      return bedrock(modelConfig.modelId);
    case "fireworks":
      return fireworks(modelConfig.modelId);
    case "together":
      return together(modelConfig.modelId);
    case "perplexity":
      return perplexity(modelConfig.modelId);
    case "ollama":
      return ollama(modelConfig.modelId);
    default:
      return null;
  }
}

export const providerList = Object.values(providers);
export const defaultProvider: ProviderId = "groq";
export const defaultModel = "llama-3.1-8b-instant";
