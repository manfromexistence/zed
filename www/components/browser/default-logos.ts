import { Discord } from "@/components/ui/svgs/discord";
import { Figma } from "@/components/ui/svgs/figma";
import { GithubDark } from "@/components/ui/svgs/githubDark";
import { NextjsIconDark } from "@/components/ui/svgs/nextjsIconDark";
import { Notion } from "@/components/ui/svgs/notion";
import { ReactLight } from "@/components/ui/svgs/reactLight";
import { Slack } from "@/components/ui/svgs/slack";
import { Stripe } from "@/components/ui/svgs/stripe";
import { Supabase } from "@/components/ui/svgs/supabase";
import { Typescript } from "@/components/ui/svgs/typescript";
import { VercelThemed } from "@/components/ui/svgs/vercelThemed";
import { Youtube } from "@/components/ui/svgs/youtube";
import type { SVGLogo } from "./types";

export const DEFAULT_LOGOS: SVGLogo[] = [
  { id: 1, title: "React", component: ReactLight },
  { id: 2, title: "Next.js", component: NextjsIconDark },
  { id: 3, title: "TypeScript", component: Typescript },
  { id: 4, title: "GitHub", component: GithubDark },
  { id: 5, title: "Vercel", component: VercelThemed },
  { id: 6, title: "Figma", component: Figma },
  { id: 7, title: "Notion", component: Notion },
  { id: 8, title: "Slack", component: Slack },
  { id: 9, title: "Discord", component: Discord },
  { id: 10, title: "YouTube", component: Youtube },
  { id: 11, title: "Supabase", component: Supabase },
  { id: 12, title: "Stripe", component: Stripe },
];
