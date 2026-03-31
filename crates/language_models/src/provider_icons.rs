// Auto-generated LLM provider icon integration
// Generated on: 2026-03-31T11:00:21.983Z
//
// This file provides icon mappings for 164 LLM/AI providers in Zed
// Icons sourced from: SVGL and generated fallbacks

use gpui::{SharedString, Svg, svg};
use std::collections::HashMap;
use std::sync::OnceLock;

static ICON_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static DISPLAY_NAME_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProviderUiCategory {
    MajorCloudAi,
    MajorAiCompanies,
    InferencePlatforms,
    HardwareAccelerator,
    ChineseProviders,
    RegionalSpecialized,
    DeveloperTools,
    AudioSpeech,
    ImageVideo,
    SearchRag,
    SpecializedNiche,
    AdditionalPlatforms,
}

impl ProviderUiCategory {
    pub fn label(self) -> &'static str {
        match self {
            Self::MajorCloudAi => "Major Cloud AI",
            Self::MajorAiCompanies => "Major AI Companies",
            Self::InferencePlatforms => "Inference Platforms",
            Self::HardwareAccelerator => "Hardware / Accelerator",
            Self::ChineseProviders => "Chinese Providers",
            Self::RegionalSpecialized => "Regional / Specialized",
            Self::DeveloperTools => "Developer Tools",
            Self::AudioSpeech => "Audio / Speech",
            Self::ImageVideo => "Image / Video",
            Self::SearchRag => "Search / RAG",
            Self::SpecializedNiche => "Specialized / Niche",
            Self::AdditionalPlatforms => "Additional Platforms",
        }
    }

    pub fn sort_key(self) -> usize {
        match self {
            Self::MajorCloudAi => 0,
            Self::MajorAiCompanies => 1,
            Self::InferencePlatforms => 2,
            Self::HardwareAccelerator => 3,
            Self::ChineseProviders => 4,
            Self::RegionalSpecialized => 5,
            Self::DeveloperTools => 6,
            Self::AudioSpeech => 7,
            Self::ImageVideo => 8,
            Self::SearchRag => 9,
            Self::SpecializedNiche => 10,
            Self::AdditionalPlatforms => 11,
        }
    }
}

fn canonical_provider_id(provider_id: &str) -> &str {
    match provider_id.trim() {
        "amazon-bedrock" | "aws_bedrock" => "bedrock",
        "azure_openai" => "azure",
        "azure_ai_foundry" => "azure_ai",
        "cloudflare_ai_gateway" | "cloudflare_workers_ai" => "cloudflare",
        "copilot_chat" | "github-models" | "github_models" => "github_copilot",
        "deep_infra" => "deepinfra",
        "doubao" => "volcengine",
        "fal" => "fal_ai",
        "fireworks-ai" => "fireworks_ai",
        "friendli" => "friendliai",
        "google_gemini" => "gemini",
        "google_vertex_ai" | "vertex-ai" => "vertex_ai",
        "hugging_face" => "huggingface",
        "lite_llm_proxy" => "litellm",
        "moonshot_kimi" => "moonshot",
        "open-router" => "openrouter",
        "open_router" => "openrouter",
        "playht" => "play_ht",
        "qwen" | "qwen_alibaba" => "dashscope",
        "runway" => "runwayml",
        "text-generation-webui" => "text_generation_webui",
        "together" => "together_ai",
        "vercel" => "vercel_ai",
        "whisper-1" => "whisper",
        "x_ai" => "xai",
        "yi_01ai" => "yi",
        "zhipu_chatglm" => "zhipu",
        other => other,
    }
}

pub fn get_provider_ui_category(provider_id: &str) -> ProviderUiCategory {
    match canonical_provider_id(provider_id) {
        "openai" | "anthropic" | "gemini" | "google" | "palm" | "vertex_ai" | "azure"
        | "azure_ai" | "bedrock" | "sagemaker" => ProviderUiCategory::MajorCloudAi,
        "mistral" | "cohere" | "deepseek" | "xai" | "meta_llama" | "stability" | "ai21"
        | "writer" | "inflection" | "adept" | "character_ai" | "perplexity" | "you" | "poe" => {
            ProviderUiCategory::MajorAiCompanies
        }
        "openrouter" | "together_ai" | "replicate" | "fireworks_ai" | "anyscale" | "groq"
        | "deepinfra" | "baseten" | "modal" | "runpod" | "banana" | "beam" | "predibase"
        | "octoai" | "lepton" | "fal_ai" | "novita" | "hyperbolic" | "featherless_ai"
        | "friendliai" => ProviderUiCategory::InferencePlatforms,
        "cerebras" | "sambanova" | "graphcore" | "tenstorrent" | "nvidia_nim" | "lambda_ai"
        | "coreweave" | "vast_ai" | "paperspace" | "jarvis_labs" => {
            ProviderUiCategory::HardwareAccelerator
        }
        "dashscope" | "moonshot" | "minimax" | "baichuan" | "zhipu" | "zai" | "volcengine"
        | "sensetime" | "baidu" | "tencent" | "iflytek" | "stepfun" | "yi" | "gigachat"
        | "yandex" => ProviderUiCategory::ChineseProviders,
        "ai71" | "sarvam" | "naver" | "kakao" | "rinna" | "sakana" | "lightblue"
        | "aleph_alpha" | "ovhcloud" | "scaleway" | "nebius" | "nscale" | "oci" | "watsonx"
        | "clarifai" => ProviderUiCategory::RegionalSpecialized,
        "github_copilot"
        | "vercel_ai"
        | "huggingface"
        | "ollama"
        | "lmstudio"
        | "gpt4all"
        | "llamafile"
        | "vllm"
        | "text_generation_webui"
        | "litellm"
        | "langchain"
        | "llamaindex"
        | "haystack"
        | "semantic_kernel"
        | "autogen" => ProviderUiCategory::DeveloperTools,
        "elevenlabs" | "deepgram" | "assemblyai" | "aws_polly" | "google_tts" | "azure_speech"
        | "whisper" | "speechify" | "resemble" | "play_ht" => ProviderUiCategory::AudioSpeech,
        "midjourney" | "dalle" | "stable_diffusion" | "black_forest_labs" | "runwayml" | "pika"
        | "synthesia" | "heygen" | "leonardo" | "ideogram" | "recraft" | "imagen" | "firefly"
        | "canva" | "clipdrop" => ProviderUiCategory::ImageVideo,
        "serper" | "tavily" | "exa_ai" | "brave_search" | "searxng" | "duckduckgo"
        | "firecrawl" | "jina_ai" | "voyage" | "pinecone" => ProviderUiCategory::SearchRag,
        "codestral" | "codeium" | "tabnine" | "cursor" | "continue" | "aider" | "phind"
        | "sourcegraph" | "replit" | "v0" | "bolt" | "lovable" | "gptengineer" | "magic"
        | "poolside" | "factory" | "augment" | "mutable" | "codegen" | "starcoder" => {
            ProviderUiCategory::SpecializedNiche
        }
        _ => ProviderUiCategory::AdditionalPlatforms,
    }
}

/// Get the icon path for a given LLM provider ID
pub fn get_provider_icon_path(provider_id: &str) -> Option<&'static str> {
    let provider_id = canonical_provider_id(provider_id);
    let map = ICON_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("adept", "icons/llm_providers/adept.svg");
        m.insert("ai21", "icons/llm_providers/ai21.svg");
        m.insert("ai71", "icons/llm_providers/ai71.svg");
        m.insert("aider", "icons/llm_providers/aider.svg");
        m.insert("aleph_alpha", "icons/llm_providers/aleph_alpha.svg");
        m.insert("anthropic", "icons/llm_providers/anthropic.svg");
        m.insert("anyscale", "icons/llm_providers/anyscale.svg");
        m.insert("assemblyai", "icons/llm_providers/assemblyai.svg");
        m.insert("augment", "icons/llm_providers/augment.svg");
        m.insert("autogen", "icons/llm_providers/autogen.svg");
        m.insert("aws_polly", "icons/llm_providers/aws_polly.svg");
        m.insert("azure", "icons/llm_providers/azure.svg");
        m.insert("azure_ai", "icons/llm_providers/azure_ai.svg");
        m.insert("azure_speech", "icons/llm_providers/azure_speech.svg");
        m.insert("baichuan", "icons/llm_providers/baichuan.svg");
        m.insert("baidu", "icons/llm_providers/baidu.svg");
        m.insert("banana", "icons/llm_providers/banana.svg");
        m.insert("baseten", "icons/llm_providers/baseten.svg");
        m.insert("beam", "icons/llm_providers/beam.svg");
        m.insert("bedrock", "icons/llm_providers/bedrock.svg");
        m.insert(
            "black_forest_labs",
            "icons/llm_providers/black_forest_labs.svg",
        );
        m.insert("bolt", "icons/llm_providers/bolt.svg");
        m.insert("brave_search", "icons/llm_providers/brave_search.svg");
        m.insert("canva", "icons/llm_providers/canva.svg");
        m.insert("cerebras", "icons/llm_providers/cerebras.svg");
        m.insert("character_ai", "icons/llm_providers/character_ai.svg");
        m.insert("clarifai", "icons/llm_providers/clarifai.svg");
        m.insert("clipdrop", "icons/llm_providers/clipdrop.svg");
        m.insert("cloudflare", "icons/llm_providers/cloudflare.svg");
        m.insert("codegen", "icons/llm_providers/codegen.svg");
        m.insert("codeium", "icons/llm_providers/codeium.svg");
        m.insert("codestral", "icons/llm_providers/codestral.svg");
        m.insert("cohere", "icons/llm_providers/cohere.svg");
        m.insert("comet", "icons/llm_providers/comet.svg");
        m.insert("continue", "icons/llm_providers/continue.svg");
        m.insert("coreweave", "icons/llm_providers/coreweave.svg");
        m.insert("cursor", "icons/llm_providers/cursor.svg");
        m.insert("dalle", "icons/llm_providers/dalle.svg");
        m.insert("dashscope", "icons/llm_providers/dashscope.svg");
        m.insert("databricks", "icons/llm_providers/databricks.svg");
        m.insert("datarobot", "icons/llm_providers/datarobot.svg");
        m.insert("deepgram", "icons/llm_providers/deepgram.svg");
        m.insert("deepinfra", "icons/llm_providers/deepinfra.svg");
        m.insert("deepseek", "icons/llm_providers/deepseek.svg");
        m.insert("domino", "icons/llm_providers/domino.svg");
        m.insert("duckduckgo", "icons/llm_providers/duckduckgo.svg");
        m.insert("elevenlabs", "icons/llm_providers/elevenlabs.svg");
        m.insert("exa_ai", "icons/llm_providers/exa_ai.svg");
        m.insert("factory", "icons/llm_providers/factory.svg");
        m.insert("fal_ai", "icons/llm_providers/fal_ai.svg");
        m.insert("featherless_ai", "icons/llm_providers/featherless_ai.svg");
        m.insert("firecrawl", "icons/llm_providers/firecrawl.svg");
        m.insert("firefly", "icons/llm_providers/firefly.svg");
        m.insert("fireworks_ai", "icons/llm_providers/fireworks_ai.svg");
        m.insert("friendliai", "icons/llm_providers/friendliai.svg");
        m.insert("gemini", "icons/llm_providers/gemini.svg");
        m.insert("gigachat", "icons/llm_providers/gigachat.svg");
        m.insert("github_copilot", "icons/llm_providers/github_copilot.svg");
        m.insert("google", "icons/llm_providers/google.svg");
        m.insert("google_tts", "icons/llm_providers/google_tts.svg");
        m.insert("gpt4all", "icons/llm_providers/gpt4all.svg");
        m.insert("gptengineer", "icons/llm_providers/gptengineer.svg");
        m.insert("gradient_ai", "icons/llm_providers/gradient_ai.svg");
        m.insert("graphcore", "icons/llm_providers/graphcore.svg");
        m.insert("groq", "icons/llm_providers/groq.svg");
        m.insert("h2o", "icons/llm_providers/h2o.svg");
        m.insert("haystack", "icons/llm_providers/haystack.svg");
        m.insert("heroku", "icons/llm_providers/heroku.svg");
        m.insert("heygen", "icons/llm_providers/heygen.svg");
        m.insert("huggingface", "icons/llm_providers/huggingface.svg");
        m.insert("hyperbolic", "icons/llm_providers/hyperbolic.svg");
        m.insert("ideogram", "icons/llm_providers/ideogram.svg");
        m.insert("iflytek", "icons/llm_providers/iflytek.svg");
        m.insert("imagen", "icons/llm_providers/imagen.svg");
        m.insert("inflection", "icons/llm_providers/inflection.svg");
        m.insert("jarvis_labs", "icons/llm_providers/jarvis_labs.svg");
        m.insert("jina_ai", "icons/llm_providers/jina_ai.svg");
        m.insert("kakao", "icons/llm_providers/kakao.svg");
        m.insert("lambda_ai", "icons/llm_providers/lambda_ai.svg");
        m.insert("langchain", "icons/llm_providers/langchain.svg");
        m.insert("leonardo", "icons/llm_providers/leonardo.svg");
        m.insert("lepton", "icons/llm_providers/lepton.svg");
        m.insert("lightblue", "icons/llm_providers/lightblue.svg");
        m.insert("litellm", "icons/llm_providers/litellm.svg");
        m.insert("llamafile", "icons/llm_providers/llamafile.svg");
        m.insert("llamaindex", "icons/llm_providers/llamaindex.svg");
        m.insert("lmstudio", "icons/llm_providers/lmstudio.svg");
        m.insert("lovable", "icons/llm_providers/lovable.svg");
        m.insert("magic", "icons/llm_providers/magic.svg");
        m.insert("meta_llama", "icons/llm_providers/meta_llama.svg");
        m.insert("midjourney", "icons/llm_providers/midjourney.svg");
        m.insert("minimax", "icons/llm_providers/minimax.svg");
        m.insert("mistral", "icons/llm_providers/mistral.svg");
        m.insert("modal", "icons/llm_providers/modal.svg");
        m.insert("moonshot", "icons/llm_providers/moonshot.svg");
        m.insert("mutable", "icons/llm_providers/mutable.svg");
        m.insert("naver", "icons/llm_providers/naver.svg");
        m.insert("nebius", "icons/llm_providers/nebius.svg");
        m.insert("novita", "icons/llm_providers/novita.svg");
        m.insert("nscale", "icons/llm_providers/nscale.svg");
        m.insert("nvidia_nim", "icons/llm_providers/nvidia_nim.svg");
        m.insert("oci", "icons/llm_providers/oci.svg");
        m.insert("octoai", "icons/llm_providers/octoai.svg");
        m.insert("ollama", "icons/llm_providers/ollama.svg");
        m.insert("openai", "icons/llm_providers/openai.svg");
        m.insert("openrouter", "icons/llm_providers/openrouter.svg");
        m.insert("ovhcloud", "icons/llm_providers/ovhcloud.svg");
        m.insert("palm", "icons/llm_providers/palm.svg");
        m.insert("paperspace", "icons/llm_providers/paperspace.svg");
        m.insert("perplexity", "icons/llm_providers/perplexity.svg");
        m.insert("phind", "icons/llm_providers/phind.svg");
        m.insert("pika", "icons/llm_providers/pika.svg");
        m.insert("pinecone", "icons/llm_providers/pinecone.svg");
        m.insert("play_ht", "icons/llm_providers/play_ht.svg");
        m.insert("poe", "icons/llm_providers/poe.svg");
        m.insert("poolside", "icons/llm_providers/poolside.svg");
        m.insert("predibase", "icons/llm_providers/predibase.svg");
        m.insert("recraft", "icons/llm_providers/recraft.svg");
        m.insert("replicate", "icons/llm_providers/replicate.svg");
        m.insert("replit", "icons/llm_providers/replit.svg");
        m.insert("resemble", "icons/llm_providers/resemble.svg");
        m.insert("rinna", "icons/llm_providers/rinna.svg");
        m.insert("runpod", "icons/llm_providers/runpod.svg");
        m.insert("runwayml", "icons/llm_providers/runwayml.svg");
        m.insert("sagemaker", "icons/llm_providers/sagemaker.svg");
        m.insert("sakana", "icons/llm_providers/sakana.svg");
        m.insert("sambanova", "icons/llm_providers/sambanova.svg");
        m.insert("sarvam", "icons/llm_providers/sarvam.svg");
        m.insert("scaleway", "icons/llm_providers/scaleway.svg");
        m.insert("searxng", "icons/llm_providers/searxng.svg");
        m.insert("semantic_kernel", "icons/llm_providers/semantic_kernel.svg");
        m.insert("sensetime", "icons/llm_providers/sensetime.svg");
        m.insert("serper", "icons/llm_providers/serper.svg");
        m.insert("snowflake", "icons/llm_providers/snowflake.svg");
        m.insert("sourcegraph", "icons/llm_providers/sourcegraph.svg");
        m.insert("speechify", "icons/llm_providers/speechify.svg");
        m.insert("stability", "icons/llm_providers/stability.svg");
        m.insert(
            "stable_diffusion",
            "icons/llm_providers/stable_diffusion.svg",
        );
        m.insert("starcoder", "icons/llm_providers/starcoder.svg");
        m.insert("stepfun", "icons/llm_providers/stepfun.svg");
        m.insert("synthesia", "icons/llm_providers/synthesia.svg");
        m.insert("tabnine", "icons/llm_providers/tabnine.svg");
        m.insert("tavily", "icons/llm_providers/tavily.svg");
        m.insert("tencent", "icons/llm_providers/tencent.svg");
        m.insert("tenstorrent", "icons/llm_providers/tenstorrent.svg");
        m.insert(
            "text_generation_webui",
            "icons/llm_providers/text_generation_webui.svg",
        );
        m.insert("together_ai", "icons/llm_providers/together_ai.svg");
        m.insert("v0", "icons/llm_providers/v0.svg");
        m.insert("vast_ai", "icons/llm_providers/vast_ai.svg");
        m.insert("vercel_ai", "icons/llm_providers/vercel_ai.svg");
        m.insert("vertex_ai", "icons/llm_providers/vertex_ai.svg");
        m.insert("vllm", "icons/llm_providers/vllm.svg");
        m.insert("volcengine", "icons/llm_providers/volcengine.svg");
        m.insert("voyage", "icons/llm_providers/voyage.svg");
        m.insert("wandb", "icons/llm_providers/wandb.svg");
        m.insert("watsonx", "icons/llm_providers/watsonx.svg");
        m.insert("whisper", "icons/llm_providers/whisper.svg");
        m.insert("writer", "icons/llm_providers/writer.svg");
        m.insert("xai", "icons/llm_providers/xai.svg");
        m.insert("yandex", "icons/llm_providers/yandex.svg");
        m.insert("yi", "icons/llm_providers/yi.svg");
        m.insert("you", "icons/llm_providers/you.svg");
        m.insert("zai", "icons/llm_providers/zai.svg");
        m.insert("zhipu", "icons/llm_providers/zhipu.svg");
        m
    });
    map.get(provider_id).copied()
}

/// Get the display name for a given LLM provider ID
pub fn get_provider_display_name(provider_id: &str) -> Option<&'static str> {
    match provider_id.trim() {
        "github-models" | "github_models" => return Some("GitHub Models"),
        "copilot_chat" => return Some("GitHub Copilot"),
        "cloudflare_ai_gateway" => return Some("Cloudflare AI Gateway"),
        "cloudflare_workers_ai" => return Some("Cloudflare Workers AI"),
        "azure_ai_foundry" => return Some("Azure AI Foundry"),
        _ => {}
    }
    let provider_id = canonical_provider_id(provider_id);
    let map = DISPLAY_NAME_MAP.get_or_init(|| {
        let mut m = HashMap::new();
        m.insert("adept", "Adept AI");
        m.insert("ai21", "AI21 Labs");
        m.insert("ai71", "AI71 (UAE)");
        m.insert("aider", "Aider");
        m.insert("aleph_alpha", "Aleph Alpha (EU)");
        m.insert("anthropic", "Anthropic");
        m.insert("anyscale", "Anyscale");
        m.insert("assemblyai", "AssemblyAI");
        m.insert("augment", "Augment Code");
        m.insert("autogen", "AutoGen");
        m.insert("aws_polly", "AWS Polly");
        m.insert("azure", "Azure OpenAI");
        m.insert("azure_ai", "Azure AI");
        m.insert("azure_speech", "Azure Speech");
        m.insert("baichuan", "Baichuan AI");
        m.insert("baidu", "Baidu (ERNIE)");
        m.insert("banana", "Banana");
        m.insert("baseten", "Baseten");
        m.insert("beam", "Beam");
        m.insert("bedrock", "AWS Bedrock");
        m.insert("black_forest_labs", "Black Forest Labs (FLUX)");
        m.insert("bolt", "Bolt.new");
        m.insert("brave_search", "Brave Search API");
        m.insert("canva", "Canva AI");
        m.insert("cerebras", "Cerebras");
        m.insert("character_ai", "Character.AI");
        m.insert("clarifai", "Clarifai");
        m.insert("clipdrop", "ClipDrop");
        m.insert("cloudflare", "Cloudflare AI");
        m.insert("codegen", "CodeGen");
        m.insert("codeium", "Codeium");
        m.insert("codestral", "Codestral");
        m.insert("cohere", "Cohere");
        m.insert("comet", "Comet ML");
        m.insert("continue", "Continue");
        m.insert("coreweave", "CoreWeave");
        m.insert("cursor", "Cursor");
        m.insert("dalle", "DALL-E");
        m.insert("dashscope", "DashScope (Alibaba)");
        m.insert("databricks", "Databricks");
        m.insert("datarobot", "DataRobot");
        m.insert("deepgram", "Deepgram");
        m.insert("deepinfra", "DeepInfra");
        m.insert("deepseek", "DeepSeek");
        m.insert("domino", "Domino Data Lab");
        m.insert("duckduckgo", "DuckDuckGo");
        m.insert("elevenlabs", "ElevenLabs");
        m.insert("exa_ai", "Exa AI");
        m.insert("factory", "Factory AI");
        m.insert("fal_ai", "Fal AI");
        m.insert("featherless_ai", "Featherless AI");
        m.insert("firecrawl", "Firecrawl");
        m.insert("firefly", "Adobe Firefly");
        m.insert("fireworks_ai", "Fireworks AI");
        m.insert("friendliai", "FriendliAI");
        m.insert("gemini", "Google Gemini");
        m.insert("gigachat", "GigaChat (Sber)");
        m.insert("github_copilot", "GitHub Copilot");
        m.insert("google", "Google AI");
        m.insert("google_tts", "Google Text-to-Speech");
        m.insert("gpt4all", "GPT4All");
        m.insert("gptengineer", "GPT Engineer");
        m.insert("gradient_ai", "Gradient AI");
        m.insert("graphcore", "Graphcore");
        m.insert("groq", "Groq");
        m.insert("h2o", "H2O.ai");
        m.insert("haystack", "Haystack");
        m.insert("heroku", "Heroku AI");
        m.insert("heygen", "HeyGen");
        m.insert("huggingface", "Hugging Face");
        m.insert("hyperbolic", "Hyperbolic");
        m.insert("ideogram", "Ideogram");
        m.insert("iflytek", "iFlytek (Spark)");
        m.insert("imagen", "Google Imagen");
        m.insert("inflection", "Inflection AI");
        m.insert("jarvis_labs", "Jarvis Labs");
        m.insert("jina_ai", "Jina AI");
        m.insert("kakao", "Kakao (KoGPT)");
        m.insert("lambda_ai", "Lambda Labs");
        m.insert("langchain", "LangChain");
        m.insert("leonardo", "Leonardo.AI");
        m.insert("lepton", "Lepton AI");
        m.insert("lightblue", "LightBlue (Japan)");
        m.insert("litellm", "LiteLLM Proxy");
        m.insert("llamafile", "Llamafile");
        m.insert("llamaindex", "LlamaIndex");
        m.insert("lmstudio", "LM Studio");
        m.insert("lovable", "Lovable");
        m.insert("magic", "Magic.dev");
        m.insert("meta_llama", "Meta Llama");
        m.insert("midjourney", "Midjourney");
        m.insert("minimax", "MiniMax");
        m.insert("mistral", "Mistral AI");
        m.insert("modal", "Modal");
        m.insert("moonshot", "Moonshot AI");
        m.insert("mutable", "Mutable AI");
        m.insert("naver", "Naver (HyperCLOVA)");
        m.insert("nebius", "Nebius (EU)");
        m.insert("novita", "Novita AI");
        m.insert("nscale", "nScale (EU)");
        m.insert("nvidia_nim", "NVIDIA NIM");
        m.insert("oci", "Oracle Cloud");
        m.insert("octoai", "OctoAI");
        m.insert("ollama", "Ollama");
        m.insert("openai", "OpenAI");
        m.insert("openrouter", "OpenRouter");
        m.insert("ovhcloud", "OVHcloud (EU)");
        m.insert("palm", "Google PaLM");
        m.insert("paperspace", "Paperspace");
        m.insert("perplexity", "Perplexity");
        m.insert("phind", "Phind");
        m.insert("pika", "Pika Labs");
        m.insert("pinecone", "Pinecone");
        m.insert("play_ht", "Play.ht");
        m.insert("poe", "Poe (Quora)");
        m.insert("poolside", "Poolside");
        m.insert("predibase", "Predibase");
        m.insert("recraft", "Recraft");
        m.insert("replicate", "Replicate");
        m.insert("replit", "Replit AI");
        m.insert("resemble", "Resemble AI");
        m.insert("rinna", "Rinna (Japan)");
        m.insert("runpod", "RunPod");
        m.insert("runwayml", "Runway ML");
        m.insert("sagemaker", "AWS SageMaker");
        m.insert("sakana", "Sakana AI");
        m.insert("sambanova", "SambaNova");
        m.insert("sarvam", "Sarvam AI (India)");
        m.insert("scaleway", "Scaleway (EU)");
        m.insert("searxng", "SearXNG");
        m.insert("semantic_kernel", "Semantic Kernel");
        m.insert("sensetime", "SenseTime");
        m.insert("serper", "Serper");
        m.insert("snowflake", "Snowflake Cortex");
        m.insert("sourcegraph", "Sourcegraph Cody");
        m.insert("speechify", "Speechify");
        m.insert("stability", "Stability AI");
        m.insert("stable_diffusion", "Stable Diffusion");
        m.insert("starcoder", "StarCoder");
        m.insert("stepfun", "StepFun");
        m.insert("synthesia", "Synthesia");
        m.insert("tabnine", "Tabnine");
        m.insert("tavily", "Tavily");
        m.insert("tencent", "Tencent (Hunyuan)");
        m.insert("tenstorrent", "Tenstorrent");
        m.insert("text_generation_webui", "Text Generation WebUI");
        m.insert("together_ai", "Together AI");
        m.insert("v0", "v0 (Vercel)");
        m.insert("vast_ai", "Vast.ai");
        m.insert("vercel_ai", "Vercel AI");
        m.insert("vertex_ai", "Google Vertex AI");
        m.insert("vllm", "vLLM");
        m.insert("volcengine", "VolcEngine (ByteDance)");
        m.insert("voyage", "Voyage AI");
        m.insert("wandb", "Weights & Biases");
        m.insert("watsonx", "IBM watsonx");
        m.insert("whisper", "OpenAI Whisper");
        m.insert("writer", "Writer");
        m.insert("xai", "xAI (Grok)");
        m.insert("yandex", "Yandex GPT");
        m.insert("yi", "Yi (01.AI)");
        m.insert("you", "You.com");
        m.insert("zai", "Z.AI");
        m.insert("zhipu", "Zhipu AI (GLM)");
        m
    });
    map.get(provider_id).copied()
}

/// Load an SVG icon for a given LLM provider
pub fn load_provider_icon(provider_id: &str) -> Option<Svg> {
    get_provider_icon_path(provider_id).map(|path| svg().path(SharedString::from(path)))
}

/// Get all available provider IDs
pub fn get_all_provider_ids() -> Vec<&'static str> {
    vec![
        "adept",
        "ai21",
        "ai71",
        "aider",
        "aleph_alpha",
        "anthropic",
        "anyscale",
        "assemblyai",
        "augment",
        "autogen",
        "aws_polly",
        "azure",
        "azure_ai",
        "azure_speech",
        "baichuan",
        "baidu",
        "banana",
        "baseten",
        "beam",
        "bedrock",
        "black_forest_labs",
        "bolt",
        "brave_search",
        "canva",
        "cerebras",
        "character_ai",
        "clarifai",
        "clipdrop",
        "cloudflare",
        "codegen",
        "codeium",
        "codestral",
        "cohere",
        "comet",
        "continue",
        "coreweave",
        "cursor",
        "dalle",
        "dashscope",
        "databricks",
        "datarobot",
        "deepgram",
        "deepinfra",
        "deepseek",
        "domino",
        "duckduckgo",
        "elevenlabs",
        "exa_ai",
        "factory",
        "fal_ai",
        "featherless_ai",
        "firecrawl",
        "firefly",
        "fireworks_ai",
        "friendliai",
        "gemini",
        "gigachat",
        "github_copilot",
        "google",
        "google_tts",
        "gpt4all",
        "gptengineer",
        "gradient_ai",
        "graphcore",
        "groq",
        "h2o",
        "haystack",
        "heroku",
        "heygen",
        "huggingface",
        "hyperbolic",
        "ideogram",
        "iflytek",
        "imagen",
        "inflection",
        "jarvis_labs",
        "jina_ai",
        "kakao",
        "lambda_ai",
        "langchain",
        "leonardo",
        "lepton",
        "lightblue",
        "litellm",
        "llamafile",
        "llamaindex",
        "lmstudio",
        "lovable",
        "magic",
        "meta_llama",
        "midjourney",
        "minimax",
        "mistral",
        "modal",
        "moonshot",
        "mutable",
        "naver",
        "nebius",
        "novita",
        "nscale",
        "nvidia_nim",
        "oci",
        "octoai",
        "ollama",
        "openai",
        "openrouter",
        "ovhcloud",
        "palm",
        "paperspace",
        "perplexity",
        "phind",
        "pika",
        "pinecone",
        "play_ht",
        "poe",
        "poolside",
        "predibase",
        "recraft",
        "replicate",
        "replit",
        "resemble",
        "rinna",
        "runpod",
        "runwayml",
        "sagemaker",
        "sakana",
        "sambanova",
        "sarvam",
        "scaleway",
        "searxng",
        "semantic_kernel",
        "sensetime",
        "serper",
        "snowflake",
        "sourcegraph",
        "speechify",
        "stability",
        "stable_diffusion",
        "starcoder",
        "stepfun",
        "synthesia",
        "tabnine",
        "tavily",
        "tencent",
        "tenstorrent",
        "text_generation_webui",
        "together_ai",
        "v0",
        "vast_ai",
        "vercel_ai",
        "vertex_ai",
        "vllm",
        "volcengine",
        "voyage",
        "wandb",
        "watsonx",
        "whisper",
        "writer",
        "xai",
        "yandex",
        "yi",
        "you",
        "zai",
        "zhipu",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_providers_have_icons() {
        let providers = get_all_provider_ids();
        assert_eq!(providers.len(), 164);

        for provider in providers {
            assert!(get_provider_icon_path(provider).is_some());
            assert!(get_provider_display_name(provider).is_some());
        }
    }

    #[test]
    fn test_provider_aliases_resolve() {
        assert_eq!(
            get_provider_icon_path("github-models"),
            get_provider_icon_path("github_copilot")
        );
        assert_eq!(
            get_provider_icon_path("together"),
            get_provider_icon_path("together_ai")
        );
        assert_eq!(
            get_provider_display_name("github-models"),
            Some("GitHub Models")
        );
    }
}
