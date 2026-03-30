# 🚀 Complete Guide: Vercel AI SDK — All Provider NPM Packages

Since you're using the **Vercel AI SDK** and want to support **100+ providers**, here's the good news — you have **multiple strategies** to achieve this, from single-package solutions to individual first-party and community provider packages.

---

## ⚡ Strategy 1: Fastest Way — Use the **Vercel AI Gateway** (100+ models, 1 package!)

The AI Gateway provider connects you to models from multiple AI providers through a single interface. Instead of integrating with each provider separately, you can access OpenAI, Anthropic, Google, Meta, xAI, and other providers and their models.

The AI Gateway supports models from OpenAI, Anthropic, Google, Meta, xAI, Mistral, DeepSeek, Amazon Bedrock, Cohere, Perplexity, Alibaba, and other providers.

AI Gateway supports routing to 20+ providers.

```bash
npm install ai
```

```ts
import { generateText } from 'ai';

// Just pass a string — the AI Gateway routes automatically!
const result = await generateText({
  model: 'anthropic/claude-opus-4.6',  // or 'openai/gpt-5.4', 'google/gemini-3-flash', etc.
  prompt: 'Hello!',
});
```

Access models from multiple providers without having to install additional provider modules/dependencies.

---

## ⚡ Strategy 2: Use **OpenRouter** (300+ models, 1 package!)

The OpenRouter provider for the Vercel AI SDK gives access to over 300 large language models on the OpenRouter chat and completion APIs.

```bash
npm install @openrouter/ai-sdk-provider
```

```ts
import { openrouter } from '@openrouter/ai-sdk-provider';
import { generateText } from 'ai';

const { text } = await generateText({
  model: openrouter('openai/gpt-4o'),
  prompt: 'Write a vegetarian lasagna recipe for 4 people.',
});
```

There are 262 other projects in the npm registry using @openrouter/ai-sdk-provider.

---

## 📦 Strategy 3: Install Individual **Official (First-Party)** `@ai-sdk/*` Providers

The AI SDK provides a unified API to interact with model providers like OpenAI, Anthropic, Google, and more.

Here is the **full list of official first-party provider packages**:

| # | npm Package | Provider | Weekly Downloads |
|---|------------|----------|-----------------|
| 1 | `ai` | **Core SDK + AI Gateway** | over 20 million monthly downloads |
| 2 | `@ai-sdk/openai` | OpenAI chat/completion APIs and embeddings | 2,133,843 |
| 3 | `@ai-sdk/anthropic` | Anthropic Messages API | 1,348 dependents |
| 4 | `@ai-sdk/google` | Google Generative AI | 1,813,848 |
| 5 | `@ai-sdk/google-vertex` | Google Vertex AI APIs | — |
| 6 | `@ai-sdk/azure` | Azure OpenAI API | — |
| 7 | `@ai-sdk/xai` | xAI chat and completion APIs (Grok) | 472,525 |
| 8 | `@ai-sdk/mistral` | Mistral chat API | — |
| 9 | `@ai-sdk/cohere` | Cohere API | — |
| 10 | `@ai-sdk/amazon-bedrock` | Amazon Bedrock | — |
| 11 | `@ai-sdk/vercel` | Vercel v0 API for building modern web applications | — |
| 12 | `@ai-sdk/openai-compatible` | Foundation for implementing providers that expose an OpenAI-compatible API |  — |

### OpenAI-Compatible Providers (via `@ai-sdk/openai-compatible` or `@ai-sdk/openai`)

OpenAI-Compatible Providers include xAI, Fireworks, Cerebras, TogetherAI, DeepInfra, and DeepSeek.

| # | Provider | How to Use |
|---|----------|------------|
| 13 | **Fireworks AI** | `@ai-sdk/openai-compatible` or AI Gateway |
| 14 | **Cerebras** | `@ai-sdk/openai-compatible` |
| 15 | **Together AI** | `@ai-sdk/openai-compatible` |
| 16 | **DeepInfra** | `@ai-sdk/openai-compatible` |
| 17 | **DeepSeek** | `@ai-sdk/openai-compatible` |
| 18 | **Groq** | `@ai-sdk/openai-compatible` |
| 19 | **Perplexity** | `@ai-sdk/openai-compatible` |
| 20 | **Ollama (local)** | `@ai-sdk/openai-compatible` |

---

## 🌍 Strategy 4: **Community Providers** (npm packages)

There are also community providers that have been created using the Language Model Specification. The language model specification is published as an open-source package, which you can use to create custom providers.

| # | npm Package | Provider |
|---|------------|----------|
| 21 | `@openrouter/ai-sdk-provider` | **OpenRouter** (300+ models) |
| 22 | `@browser-ai/core` | Chrome and Edge browser's built-in AI models |
| 23 | `@browser-ai/web-llm` | Popular open-source models using the WebLLM inference engine |
| 24 | `@browser-ai/transformers-js` | Popular open-source models using Transformers.js |
| 25 | `@ai-sdk/mcp` | MCP (Model Context Protocol) — stable and available |

More community providers are listed at: **https://ai-sdk.dev/providers/community-providers**

---

## 🏗️ Strategy 5: Build a **Custom Provider**

You can build your own provider to integrate any service with the AI SDK. The AI SDK provides a Language Model Specification that ensures compatibility across providers. See Writing a Custom Provider for a complete guide.

Utility packages for building custom providers:

| npm Package | Purpose |
|------------|---------|
| `@ai-sdk/provider` | Core provider interfaces/types |
| `@ai-sdk/provider-utils` | Shared utilities for building providers |
| `@ai-sdk/openai-compatible` | Quick base for any OpenAI-compatible API |

---

## 🎯 My Recommendation for 100+ Providers

For your use case of **100+ provider support**, here's what I'd recommend:

### Install these 3 packages:

```bash
npm install ai @openrouter/ai-sdk-provider @ai-sdk/openai-compatible
```

| Package | What it Unlocks |
|---------|----------------|
| `ai` | By default, the AI SDK uses the Vercel AI Gateway to give you access to all major providers out of the box. |
| `@openrouter/ai-sdk-provider` | 300+ models from all major and niche providers |
| `@ai-sdk/openai-compatible` | Any self-hosted or custom OpenAI-compatible endpoint (Ollama, LM Studio, vLLM, etc.) |

### Then optionally add first-party packages for **deeper integration**:

```bash
npm install @ai-sdk/openai @ai-sdk/anthropic @ai-sdk/google @ai-sdk/xai @ai-sdk/mistral @ai-sdk/azure @ai-sdk/amazon-bedrock @ai-sdk/cohere @ai-sdk/google-vertex
```

This gives you **direct, type-safe, first-party support** with provider-specific features (e.g., OpenAI's cached prompts, Anthropic's extended thinking, Google's grounding).

---

## 📊 Summary Table

| Strategy | Packages Needed | # of Providers/Models |
|----------|----------------|----------------------|
| **AI Gateway** (built into `ai`) | 1 | 20+ providers, 100s of models |
| **OpenRouter** | 1 | 300+ models |
| **First-party `@ai-sdk/*`** | ~12 | 12 direct providers |
| **Community providers** | Various | Dozens more |
| **`@ai-sdk/openai-compatible`** | 1 | Any OpenAI-compatible API |
| **Custom provider** | `@ai-sdk/provider-utils` | Unlimited |

The SDK supports over 50 AI providers and models through its official and community provider packages, including local models via Ollama. And with **OpenRouter + AI Gateway**, you easily exceed **100+ providers**.

Switch between AI providers by changing a single line of code. 🎉

---

## LiteLLM Proxy

**Provider ID:** `litellm`

**Description:** Self-hosted OpenAI-compatible proxy for 100+ LLM providers (OpenAI, Anthropic, Azure, Bedrock, HuggingFace, Ollama, etc.)

**Type:** Proxy Server (requires local/remote deployment)

**Setup:**

1. Install LiteLLM:
   ```bash
   pip install 'litellm[proxy]' 'litellm[extra_proxy]'
   ```

2. Create configuration file `litellm_config.yaml`:
   ```yaml
   model_list:
     - model_name: gpt-4
       litellm_params:
         model: openai/gpt-4
         api_key: os.environ/OPENAI_API_KEY
     - model_name: claude-3
       litellm_params:
         model: anthropic/claude-3-opus-20240229
         api_key: os.environ/ANTHROPIC_API_KEY
   ```

3. Run the proxy server:
   ```bash
   litellm --config litellm_config.yaml --port 4000
   ```

4. Set environment variables:
   ```bash
   LITELLM_PROXY_URL=http://localhost:4000  # or your deployed URL
   LITELLM_API_KEY=sk-1234  # your proxy authentication key
   ```

**Benefits:**
- Unified API for 100+ providers
- Cost tracking and budgets
- Load balancing
- Caching
- Rate limiting
- Observability

**Documentation:** https://docs.litellm.ai/

**Note:** models.dev is a database/directory of AI models, not a direct API provider. Use LiteLLM or other aggregators to access multiple providers.

---

## 🔗 Additional Resources

- **models.dev API**: https://models.dev - Aggregates 100+ providers
- **LiteLLM**: https://docs.litellm.ai/docs/providers - Supports 100+ LLM providers
- **AI SDK Docs**: https://ai-sdk.dev
- **OpenRouter Models**: https://openrouter.ai/models
- **Community Providers**: https://ai-sdk.dev/providers/community-providers
