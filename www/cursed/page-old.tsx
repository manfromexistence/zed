"use client";

import {
  closestCenter,
  DndContext,
  type DragEndEvent,
  type DragOverEvent,
  DragOverlay,
  type DragStartEvent,
  PointerSensor,
  useSensor,
  useSensors,
} from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import { VisuallyHidden } from "@radix-ui/react-visually-hidden";
import * as LucideIcons from "lucide-react";
import {
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  CircleDashed,
  Cog,
  Columns2,
  Copy,
  Folder,
  FolderOpen,
  Grid3x3,
  Link,
  MoreHorizontal,
  PanelLeft,
  Play,
  Plus,
  Search,
  ShieldAlert,
  ShieldBan,
  SkipBack,
  SkipForward,
  Trash2,
  Volume2,
  X,
} from "lucide-react";
import { AnimatePresence, motion } from "motion/react";
import * as React from "react";
import { useEffect, useState } from "react";
import { ThemeSwitcher } from "@/components/theme-switcher";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Dialog, DialogContent, DialogTitle } from "@/components/ui/dialog";
import { Dock, DockIcon } from "@/components/ui/dock";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Discord } from "@/components/ui/svgs/discord";
import { Figma } from "@/components/ui/svgs/figma";
import { GithubDark } from "@/components/ui/svgs/githubDark";
import { NextjsIconDark } from "@/components/ui/svgs/nextjsIconDark";
import { Notion } from "@/components/ui/svgs/notion";
// Import local SVGL logos
import { ReactLight } from "@/components/ui/svgs/reactLight";
import { Slack } from "@/components/ui/svgs/slack";
import { Stripe } from "@/components/ui/svgs/stripe";
import { Supabase } from "@/components/ui/svgs/supabase";
import { Typescript } from "@/components/ui/svgs/typescript";
import { Vercel } from "@/components/ui/svgs/vercel";
import { Youtube } from "@/components/ui/svgs/youtube";
import { WorkspaceDialog } from "@/components/workspace-dialog";
import { cn } from "@/lib/utils";

export type IconProps = React.HTMLAttributes<SVGElement>;

export function DockDemo() {
  return (
    <div className="relative">
      <Dock iconMagnification={60} iconDistance={100}>
        <DockIcon className="bg-black/10 dark:bg-white/10">
          <Icons.gitHub className="size-full" />
        </DockIcon>
        <DockIcon className="bg-black/10 dark:bg-white/10">
          <Icons.googleDrive className="size-full" />
        </DockIcon>
        <DockIcon className="bg-black/10 dark:bg-white/10">
          <Icons.notion className="size-full" />
        </DockIcon>
        <DockIcon className="bg-black/10 dark:bg-white/10">
          <Icons.whatsapp className="size-full" />
        </DockIcon>
      </Dock>
    </div>
  );
}

const Icons = {
  gitHub: (props: IconProps) => (
    <svg viewBox="0 0 438.549 438.549" {...props}>
      <path
        fill="currentColor"
        d="M409.132 114.573c-19.608-33.596-46.205-60.194-79.798-79.8-33.598-19.607-70.277-29.408-110.063-29.408-39.781 0-76.472 9.804-110.063 29.408-33.596 19.605-60.192 46.204-79.8 79.8C9.803 148.168 0 184.854 0 224.63c0 47.78 13.94 90.745 41.827 128.906 27.884 38.164 63.906 64.572 108.063 79.227 5.14.954 8.945.283 11.419-1.996 2.475-2.282 3.711-5.14 3.711-8.562 0-.571-.049-5.708-.144-15.417a2549.81 2549.81 0 01-.144-25.406l-6.567 1.136c-4.187.767-9.469 1.092-15.846 1-6.374-.089-12.991-.757-19.842-1.999-6.854-1.231-13.229-4.086-19.13-8.559-5.898-4.473-10.085-10.328-12.56-17.556l-2.855-6.57c-1.903-4.374-4.899-9.233-8.992-14.559-4.093-5.331-8.232-8.945-12.419-10.848l-1.999-1.431c-1.332-.951-2.568-2.098-3.711-3.429-1.142-1.331-1.997-2.663-2.568-3.997-.572-1.335-.098-2.43 1.427-3.289 1.525-.859 4.281-1.276 8.28-1.276l5.708.853c3.807.763 8.516 3.042 14.133 6.851 5.614 3.806 10.229 8.754 13.846 14.842 4.38 7.806 9.657 13.754 15.846 17.847 6.184 4.093 12.419 6.136 18.699 6.136 6.28 0 11.704-.476 16.274-1.423 4.565-.952 8.848-2.383 12.847-4.285 1.713-12.758 6.377-22.559 13.988-29.41-10.848-1.14-20.601-2.857-29.264-5.14-8.658-2.286-17.605-5.996-26.835-11.14-9.235-5.137-16.896-11.516-22.985-19.126-6.09-7.614-11.088-17.61-14.987-29.979-3.901-12.374-5.852-26.648-5.852-42.826 0-23.035 7.52-42.637 22.557-58.817-7.044-17.318-6.379-36.732 1.997-58.24 5.52-1.715 13.706-.428 24.554 3.853 10.85 4.283 18.794 7.952 23.84 10.994 5.046 3.041 9.089 5.618 12.135 7.708 17.705-4.947 35.976-7.421 54.818-7.421s37.117 2.474 54.823 7.421l10.849-6.849c7.419-4.57 16.18-8.758 26.262-12.565 10.088-3.805 17.802-4.853 23.134-3.138 8.562 21.509 9.325 40.922 2.279 58.24 15.036 16.18 22.559 35.787 22.559 58.817 0 16.178-1.958 30.497-5.853 42.966-3.9 12.471-8.941 22.457-15.125 29.979-6.191 7.521-13.901 13.85-23.131 18.986-9.232 5.14-18.182 8.85-26.84 11.136-8.662 2.286-18.415 4.004-29.263 5.146 9.894 8.562 14.842 22.077 14.842 40.539v60.237c0 3.422 1.19 6.279 3.572 8.562 2.379 2.279 6.136 2.95 11.276 1.995 44.163-14.653 80.185-41.062 108.068-79.226 27.88-38.161 41.825-81.126 41.825-128.906-.01-39.771-9.818-76.454-29.414-110.049z"
      ></path>
    </svg>
  ),
  notion: (props: IconProps) => (
    <svg
      width="100"
      height="100"
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      {...props}
    >
      <path
        d="M6.017 4.313l55.333 -4.087c6.797 -0.583 8.543 -0.19 12.817 2.917l17.663 12.443c2.913 2.14 3.883 2.723 3.883 5.053v68.243c0 4.277 -1.553 6.807 -6.99 7.193L24.467 99.967c-4.08 0.193 -6.023 -0.39 -8.16 -3.113L3.3 79.94c-2.333 -3.113 -3.3 -5.443 -3.3 -8.167V11.113c0 -3.497 1.553 -6.413 6.017 -6.8z"
        fill="#fff"
      />
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M61.35 0.227l-55.333 4.087C1.553 4.7 0 7.617 0 11.113v60.66c0 2.723 0.967 5.053 3.3 8.167l13.007 16.913c2.137 2.723 4.08 3.307 8.16 3.113l64.257 -3.89c5.433 -0.387 6.99 -2.917 6.99 -7.193V20.64c0 -2.21 -0.873 -2.847 -3.443 -4.733L74.167 3.143c-4.273 -3.107 -6.02 -3.5 -12.817 -2.917zM25.92 19.523c-5.247 0.353 -6.437 0.433 -9.417 -1.99L8.927 11.507c-0.77 -0.78 -0.383 -1.753 1.557 -1.947l53.193 -3.887c4.467 -0.39 6.793 1.167 8.54 2.527l9.123 6.61c0.39 0.197 1.36 1.36 0.193 1.36l-54.933 3.307 -0.68 0.047zM19.803 88.3V30.367c0 -2.53 0.777 -3.697 3.103 -3.893L86 22.78c2.14 -0.193 3.107 1.167 3.107 3.693v57.547c0 2.53 -0.39 4.67 -3.883 4.863l-60.377 3.5c-3.493 0.193 -5.043 -0.97 -5.043 -4.083zm59.6 -54.827c0.387 1.75 0 3.5 -1.75 3.7l-2.91 0.577v42.773c-2.527 1.36 -4.853 2.137 -6.797 2.137 -3.107 0 -3.883 -0.973 -6.21 -3.887l-19.03 -29.94v28.967l6.02 1.363s0 3.5 -4.857 3.5l-13.39 0.777c-0.39 -0.78 0 -2.723 1.357 -3.11l3.497 -0.97v-38.3L30.48 40.667c-0.39 -1.75 0.58 -4.277 3.3 -4.473l14.367 -0.967 19.8 30.327v-26.83l-5.047 -0.58c-0.39 -2.143 1.163 -3.7 3.103 -3.89l13.4 -0.78z"
        fill="#000"
      />
    </svg>
  ),
  googleDrive: (props: IconProps) => (
    <svg viewBox="0 0 87.3 78" xmlns="http://www.w3.org/2000/svg" {...props}>
      <path
        d="m6.6 66.85 3.85 6.65c.8 1.4 1.95 2.5 3.3 3.3l13.75-23.8h-27.5c0 1.55.4 3.1 1.2 4.5z"
        fill="#0066da"
      />
      <path
        d="m43.65 25-13.75-23.8c-1.35.8-2.5 1.9-3.3 3.3l-25.4 44a9.06 9.06 0 0 0 -1.2 4.5h27.5z"
        fill="#00ac47"
      />
      <path
        d="m73.55 76.8c1.35-.8 2.5-1.9 3.3-3.3l1.6-2.75 7.65-13.25c.8-1.4 1.2-2.95 1.2-4.5h-27.502l5.852 11.5z"
        fill="#ea4335"
      />
      <path
        d="m43.65 25 13.75-23.8c-1.35-.8-2.9-1.2-4.5-1.2h-18.5c-1.6 0-3.15.45-4.5 1.2z"
        fill="#00832d"
      />
      <path
        d="m59.8 53h-32.3l-13.75 23.8c1.35.8 2.9 1.2 4.5 1.2h50.8c1.6 0 3.15-.45 4.5-1.2z"
        fill="#2684fc"
      />
      <path
        d="m73.4 26.5-12.7-22c-.8-1.4-1.95-2.5-3.3-3.3l-13.75 23.8 16.15 28h27.45c0-1.55-.4-3.1-1.2-4.5z"
        fill="#ffba00"
      />
    </svg>
  ),
  whatsapp: (props: IconProps) => (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 175.216 175.552"
      {...props}
    >
      <defs>
        <linearGradient
          id="b"
          x1="85.915"
          x2="86.535"
          y1="32.567"
          y2="137.092"
          gradientUnits="userSpaceOnUse"
        >
          <stop offset="0" stopColor="#57d163" />
          <stop offset="1" stopColor="#23b33a" />
        </linearGradient>
        <filter
          id="a"
          width="1.115"
          height="1.114"
          x="-.057"
          y="-.057"
          colorInterpolationFilters="sRGB"
        >
          <feGaussianBlur stdDeviation="3.531" />
        </filter>
      </defs>
      <path
        fill="#b3b3b3"
        d="m54.532 138.45 2.235 1.324c9.387 5.571 20.15 8.518 31.126 8.523h.023c33.707 0 61.139-27.426 61.153-61.135.006-16.335-6.349-31.696-17.895-43.251A60.75 60.75 0 0 0 87.94 25.983c-33.733 0-61.166 27.423-61.178 61.13a60.98 60.98 0 0 0 9.349 32.535l1.455 2.312-6.179 22.558zm-40.811 23.544L24.16 123.88c-6.438-11.154-9.825-23.808-9.821-36.772.017-40.556 33.021-73.55 73.578-73.55 19.681.01 38.154 7.669 52.047 21.572s21.537 32.383 21.53 52.037c-.018 40.553-33.027 73.553-73.578 73.553h-.032c-12.313-.005-24.412-3.094-35.159-8.954zm0 0"
        filter="url(#a)"
      />
      <path
        fill="#fff"
        d="m12.966 161.238 10.439-38.114a73.42 73.42 0 0 1-9.821-36.772c.017-40.556 33.021-73.55 73.578-73.55 19.681.01 38.154 7.669 52.047 21.572s21.537 32.383 21.53 52.037c-.018 40.553-33.027 73.553-73.578 73.553h-.032c-12.313-.005-24.412-3.094-35.159-8.954z"
      />
      <path
        fill="url(#linearGradient1780)"
        d="M87.184 25.227c-33.733 0-61.166 27.423-61.178 61.13a60.98 60.98 0 0 0 9.349 32.535l1.455 2.312-6.179 22.559 23.146-6.069 2.235 1.324c9.387 5.571 20.15 8.518 31.126 8.524h.023c33.707 0 61.14-27.426 61.153-61.135a60.75 60.75 0 0 0-17.895-43.251 60.75 60.75 0 0 0-43.235-17.929z"
      />
      <path
        fill="url(#b)"
        d="M87.184 25.227c-33.733 0-61.166 27.423-61.178 61.13a60.98 60.98 0 0 0 9.349 32.535l1.455 2.313-6.179 22.558 23.146-6.069 2.235 1.324c9.387 5.571 20.15 8.517 31.126 8.523h.023c33.707 0 61.14-27.426 61.153-61.135a60.75 60.75 0 0 0-17.895-43.251 60.75 60.75 0 0 0-43.235-17.928z"
      />
      <path
        fill="#fff"
        fillRule="evenodd"
        d="M68.772 55.603c-1.378-3.061-2.828-3.123-4.137-3.176l-3.524-.043c-1.226 0-3.218.46-4.902 2.3s-6.435 6.287-6.435 15.332 6.588 17.785 7.506 19.013 12.718 20.381 31.405 27.75c15.529 6.124 18.689 4.906 22.061 4.6s10.877-4.447 12.408-8.74 1.532-7.971 1.073-8.74-1.685-1.226-3.525-2.146-10.877-5.367-12.562-5.981-2.91-.919-4.137.921-4.746 5.979-5.819 7.206-2.144 1.381-3.984.462-7.76-2.861-14.784-9.124c-5.465-4.873-9.154-10.891-10.228-12.73s-.114-2.835.808-3.751c.825-.824 1.838-2.147 2.759-3.22s1.224-1.84 1.836-3.065.307-2.301-.153-3.22-4.032-10.011-5.666-13.647"
      />
    </svg>
  ),
};

interface SVGLogo {
  id: number;
  title: string;
  component: React.ComponentType<React.SVGProps<SVGSVGElement>>;
}

interface Tab {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  pinned?: boolean;
  workspaceId: string;
  folderId?: string | null;
}

interface TabFolder {
  id: string;
  name: string;
  collapsed: boolean;
  tabs: Tab[];
  workspaceId: string;
  parentId?: string | null;
}

interface Workspace {
  id: string;
  name: string;
  color: string;
  icon: {
    type: "emoji" | "icon" | "dot";
    value: string;
  };
}

export default function Home(): React.ReactElement {
  const [sidebarExpanded, setSidebarExpanded] = useState(true);
  const [_searchQuery, _setSearchQuery] = useState("");
  const [activeWorkspace, setActiveWorkspace] = useState("1");
  const [activeTab, setActiveTab] = useState("1");
  const [spaceCollapsed, setSpaceCollapsed] = useState(false);
  const [isSpaceAreaHovered, setIsSpaceAreaHovered] = useState(false);
  const [commandOpen, setCommandOpen] = useState(false);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [overId, setOverId] = useState<string | null>(null);
  const [isMounted, setIsMounted] = useState(false);
  const [dropPosition, setDropPosition] = useState<
    "before" | "after" | "inside" | null
  >(null);
  const [logoContainerHovered, setLogoContainerHovered] = useState(false);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
  );

  useEffect(() => {
    setIsMounted(true);
  }, []);

  // Local SVGL logos - now stateful for reordering
  const [logos, setLogos] = useState<SVGLogo[]>([
    { id: 1, title: "React", component: ReactLight },
    { id: 2, title: "Next.js", component: NextjsIconDark },
    { id: 3, title: "TypeScript", component: Typescript },
    { id: 4, title: "GitHub", component: GithubDark },
    { id: 5, title: "Vercel", component: Vercel },
    { id: 6, title: "Figma", component: Figma },
    { id: 7, title: "Notion", component: Notion },
    { id: 8, title: "Slack", component: Slack },
    { id: 9, title: "Discord", component: Discord },
    { id: 10, title: "YouTube", component: Youtube },
    { id: 11, title: "Supabase", component: Supabase },
    { id: 12, title: "Stripe", component: Stripe },
  ]);

  const [downloadsOpen, setDownloadsOpen] = useState(false);
  const [plusMenuOpen, setPlusMenuOpen] = useState(false);
  const [workspaceDialogOpen, setWorkspaceDialogOpen] = useState(false);
  const [workspaceEditMode, setWorkspaceEditMode] = useState<
    | {
        workspaceId: string;
        currentName: string;
        currentIcon: { type: "emoji" | "icon" | "dot"; value: string };
      }
    | undefined
  >(undefined);
  const [mediaPlaying, setMediaPlaying] = useState(true);
  const [mediaProgress, _setMediaProgress] = useState(33);

  const [workspaces, setWorkspaces] = useState<Workspace[]>([
    {
      id: "1",
      name: "Personal",
      color: "hsl(var(--chart-1))",
      icon: { type: "emoji", value: "😊" },
    },
    {
      id: "2",
      name: "Work",
      color: "hsl(var(--chart-2))",
      icon: { type: "icon", value: "Briefcase" },
    },
    {
      id: "3",
      name: "Development",
      color: "hsl(var(--chart-3))",
      icon: { type: "dot", value: "" },
    },
  ]);

  const COLORS = [
    "hsl(var(--chart-1))",
    "hsl(var(--chart-2))",
    "hsl(var(--chart-3))",
    "hsl(var(--chart-4))",
    "hsl(var(--chart-5))",
    "hsl(var(--primary))",
    "hsl(var(--destructive))",
    "hsl(var(--accent))",
    "hsl(var(--secondary))",
    "hsl(var(--muted))",
  ];

  const [workspaceScrollPosition, setWorkspaceScrollPosition] = useState(0);
  const MAX_VISIBLE_WORKSPACES = 5;
  const [_isScrolling, setIsScrolling] = useState(false);
  const scrollIntervalRef = React.useRef<NodeJS.Timeout | null>(null);

  const [folders, setFolders] = useState<TabFolder[]>([
    {
      id: "folder-zen-basics",
      name: "zen basics",
      collapsed: false,
      workspaceId: "1",
      parentId: null,
      tabs: [
        {
          id: "welcome-tab",
          title: "Welcome!",
          url: "about:welcome",
          workspaceId: "1",
          folderId: "folder-zen-basics",
        },
      ],
    },
  ]);

  const [looseTabs, setLooseTabs] = useState<Tab[]>([
    {
      id: "3",
      title: "Beautiful themes for shadcn/ui — tweakcn | Theme Editor",
      url: "https://tweakcn.com",
      favicon: "https://tweakcn.com/favicon.ico",
      workspaceId: "1",
      folderId: null,
    },
    {
      id: "4",
      title: "GitHub - zen-browser/desktop",
      url: "https://github.com",
      favicon: "https://github.com/favicon.ico",
      workspaceId: "1",
      folderId: null,
    },
  ]);

  function getLogoComponentForTab(
    tab: Tab,
  ): React.ComponentType<React.SVGProps<SVGSVGElement>> {
    const title = tab.title.toLowerCase();
    const url = tab.url.toLowerCase();

    // Match based on title or URL
    if (title.includes("react") || url.includes("react")) return ReactLight;
    if (title.includes("next") || url.includes("next")) return NextjsIconDark;
    if (title.includes("typescript") || url.includes("typescript"))
      return Typescript;
    if (title.includes("github") || url.includes("github")) return GithubDark;
    if (title.includes("vercel") || url.includes("vercel")) return Vercel;
    if (title.includes("figma") || url.includes("figma")) return Figma;
    if (title.includes("notion") || url.includes("notion")) return Notion;
    if (title.includes("slack") || url.includes("slack")) return Slack;
    if (title.includes("discord") || url.includes("discord")) return Discord;
    if (title.includes("youtube") || url.includes("youtube")) return Youtube;
    if (title.includes("supabase") || url.includes("supabase")) return Supabase;
    if (title.includes("stripe") || url.includes("stripe")) return Stripe;

    // Default to a generic icon based on domain
    if (url.includes(".com")) return GithubDark;
    if (url.includes(".io")) return Vercel;
    if (url.includes(".dev")) return Typescript;

    // Final fallback
    return ReactLight;
  }

  function toggleFolder(folderId: string): void {
    setFolders(
      folders.map((folder) =>
        folder.id === folderId
          ? { ...folder, collapsed: !folder.collapsed }
          : folder,
      ),
    );
  }

  function addNewTab(): void {
    const newTab: Tab = {
      id: Date.now().toString(),
      title: "New Tab",
      url: "about:blank",
      workspaceId: activeWorkspace,
      folderId: null,
    };
    setLooseTabs([...looseTabs, newTab]);
  }

  function createFolder(): void {
    const folderCount = folders.filter(
      (f) => f.workspaceId === activeWorkspace,
    ).length;
    const newFolder: TabFolder = {
      id: `folder-${Date.now()}`,
      name: `Folder ${folderCount + 1}`,
      collapsed: false,
      workspaceId: activeWorkspace,
      parentId: null,
      tabs: [],
    };
    setFolders([...folders, newFolder]);
  }

  function deleteFolder(folderId: string): void {
    const folder = folders.find((f) => f.id === folderId);
    if (folder) {
      // Move tabs back to loose tabs
      const folderTabs = folder.tabs.map((tab) => ({ ...tab, folderId: null }));
      setLooseTabs([...looseTabs, ...folderTabs]);
    }
    setFolders(folders.filter((f) => f.id !== folderId));
  }

  function handleDragStart(event: DragStartEvent): void {
    setActiveId(event.active.id as string);
  }

  function handleDragOver(event: DragOverEvent): void {
    const overId = event.over?.id as string | null;
    setOverId(overId);

    if (!overId || !event.over) {
      setDropPosition(null);
      setLogoContainerHovered(false);
      return;
    }

    // Determine if we're dropping into a folder or between items
    const overData = event.over.data.current;
    const activeData = event.active.data.current;

    // Check if hovering over logo container
    if (overId === "logo-container" || overData?.type === "logo-container") {
      // Tabs and folders can be dropped into logo container
      if (activeData?.type === "tab" || activeData?.type === "folder") {
        setLogoContainerHovered(true);
        setDropPosition(null);
      } else {
        setLogoContainerHovered(false);
        setDropPosition(null);
      }
      return;
    } else {
      setLogoContainerHovered(false);
    }

    // Check if hovering over new-tab-button area
    if (overId === "new-tab-button" || overData?.type === "new-tab-button") {
      // Logos cannot be dropped here
      if (activeData?.type === "logo") {
        setDropPosition(null);
        return;
      }
      // Any other draggable item can be dropped here - show indicator at bottom
      setDropPosition("after");
      return;
    }

    // If dragging a logo, allow dropping anywhere
    if (activeData?.type === "logo") {
      if (overData?.type === "logo") {
        // Reordering within logo container
        const overRect = event.over.rect;
        const offsetX = event.delta.x;
        if (overRect && offsetX < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else if (overData?.type === "folder") {
        setDropPosition("inside");
      } else if (overData?.type === "tab") {
        const overRect = event.over.rect;
        const offsetY = event.delta.y;
        if (overRect && offsetY < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else if (overData?.type === "space") {
        const overRect = event.over.rect;
        const offsetY = event.delta.y;
        if (overRect && offsetY < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else if (
        overId === "new-tab-button" ||
        overData?.type === "new-tab-button"
      ) {
        setDropPosition(null);
      } else {
        setDropPosition(null);
      }
      return;
    }

    if (overData?.type === "folder") {
      // Only tabs can be dropped inside folders
      if (activeData?.type === "tab") {
        setDropPosition("inside");
      } else if (activeData?.type === "folder") {
        // Folders can be reordered
        const overRect = event.over.rect;
        const offsetY = event.delta.y;
        if (overRect && offsetY < 0) {
          setDropPosition("before");
        } else {
          setDropPosition("after");
        }
      } else {
        setDropPosition(null);
      }
    } else if (overData?.type === "tab") {
      // Calculate if we should drop before or after based on mouse position
      const overRect = event.over.rect;
      const offsetY = event.delta.y;
      if (overRect && offsetY < 0) {
        setDropPosition("before");
      } else {
        setDropPosition("after");
      }
    } else if (overData?.type === "logo") {
      // Calculate if we should drop before or after based on mouse position
      const overRect = event.over.rect;
      const offsetX = event.delta.x;
      if (overRect && offsetX < 0) {
        setDropPosition("before");
      } else {
        setDropPosition("after");
      }
    } else if (overData?.type === "space") {
      // Space section can accept drops
      const overRect = event.over.rect;
      const offsetY = event.delta.y;
      if (overRect && offsetY < 0) {
        setDropPosition("before");
      } else {
        setDropPosition("after");
      }
    } else {
      setDropPosition(null);
    }
  }

  function handleDragEnd(event: DragEndEvent): void {
    const { active, over } = event;
    setActiveId(null);
    setOverId(null);
    setDropPosition(null);
    setLogoContainerHovered(false);

    if (!over || active.id === over.id) return;

    const activeId = active.id as string;
    const overId = over.id as string;
    const activeData = active.data.current;
    const overData = over.data.current;

    // Handle logo reordering within container
    if (activeData?.type === "logo" && overData?.type === "logo") {
      const oldIndex = logos.findIndex((l) => `logo-${l.id}` === activeId);
      const newIndex = logos.findIndex((l) => `logo-${l.id}` === overId);
      if (oldIndex !== -1 && newIndex !== -1 && oldIndex !== newIndex) {
        const newLogos = [...logos];
        const [removed] = newLogos.splice(oldIndex, 1);
        newLogos.splice(newIndex, 0, removed);
        setLogos(newLogos);
      }
      return;
    }

    // Handle dragging logos OUT of container - they can be dropped anywhere
    if (activeData?.type === "logo") {
      const logoId = activeId.replace("logo-", "");
      const logo = logos.find((l) => l.id === parseInt(logoId, 10));

      if (!logo) return;

      // If dropped on logo container or another logo, don't remove (reordering)
      if (
        overId === "logo-container" ||
        overData?.type === "logo-container" ||
        overData?.type === "logo"
      ) {
        return;
      }

      // Remove logo from container for any other drop target
      setLogos(logos.filter((l) => l.id !== parseInt(logoId, 10)));

      // Handle different drop targets
      if (overId === "new-tab-button" || overData?.type === "new-tab-button") {
        // Create new tab at the end of loose tabs
        const newTab: Tab = {
          id: Date.now().toString(),
          title: logo.title,
          url: "about:blank",
          workspaceId: activeWorkspace,
          folderId: null,
        };
        setLooseTabs([...looseTabs, newTab]);
      } else if (overId === "space-section" || overData?.type === "space") {
        // Create new tab in loose tabs
        const newTab: Tab = {
          id: Date.now().toString(),
          title: logo.title,
          url: "about:blank",
          workspaceId: activeWorkspace,
          folderId: null,
        };
        setLooseTabs([...looseTabs, newTab]);
      } else if (overData?.type === "folder") {
        // Add tab to folder
        const newTab: Tab = {
          id: Date.now().toString(),
          title: logo.title,
          url: "about:blank",
          workspaceId: activeWorkspace,
          folderId: overId,
        };
        setFolders(
          folders.map((f) =>
            f.id === overId ? { ...f, tabs: [...f.tabs, newTab] } : f,
          ),
        );
      } else if (overData?.type === "tab") {
        // Create tab near the target tab
        const targetTab = looseTabs.find((t) => t.id === overId);
        const targetTabFolder = folders.find((f) =>
          f.tabs.some((t) => t.id === overId),
        );

        const newTab: Tab = {
          id: Date.now().toString(),
          title: logo.title,
          url: "about:blank",
          workspaceId: activeWorkspace,
          folderId: targetTabFolder ? targetTabFolder.id : null,
        };

        if (targetTab) {
          // Add to loose tabs
          const targetIndex = looseTabs.findIndex((t) => t.id === overId);
          const newTabs = [...looseTabs];
          newTabs.splice(targetIndex + 1, 0, newTab);
          setLooseTabs(newTabs);
        } else if (targetTabFolder) {
          // Add to folder
          setFolders(
            folders.map((f) => {
              if (f.id === targetTabFolder.id) {
                const targetIndex = f.tabs.findIndex((t) => t.id === overId);
                const newTabs = [...f.tabs];
                newTabs.splice(targetIndex + 1, 0, newTab);
                return { ...f, tabs: newTabs };
              }
              return f;
            }),
          );
        }
      }
      return;
    }

    // Handle dropping tabs/folders onto logo container
    if (overId === "logo-container" || overData?.type === "logo-container") {
      // Check if logo container has space (max 12 items)
      if (logos.length >= 12) {
        alert("Logo container is full! Maximum 12 items allowed.");
        return;
      }

      // Handle folder being dropped
      if (activeData?.type === "folder") {
        const draggedFolder = folders.find((f) => f.id === activeId);
        if (draggedFolder) {
          const newLogo: SVGLogo = {
            id: Date.now(),
            title: draggedFolder.name,
            component: Figma,
          };
          setLogos([...logos, newLogo]);
          return;
        }
      }

      // Handle tab being dropped
      let draggedTab = looseTabs.find((tab) => tab.id === activeId);
      let sourceFolder: TabFolder | undefined;

      if (!draggedTab) {
        for (const folder of folders) {
          const tab = folder.tabs.find((t) => t.id === activeId);
          if (tab) {
            draggedTab = tab;
            sourceFolder = folder;
            break;
          }
        }
      }

      if (draggedTab) {
        // Create a new logo from the tab with appropriate component
        const newLogo: SVGLogo = {
          id: Date.now(),
          title: draggedTab.title,
          component: getLogoComponentForTab(draggedTab),
        };

        setLogos([...logos, newLogo]);

        // Remove the tab from its source
        if (sourceFolder) {
          setFolders(
            folders.map((folder) =>
              folder.id === sourceFolder.id
                ? {
                    ...folder,
                    tabs: folder.tabs.filter((t) => t.id !== activeId),
                  }
                : folder,
            ),
          );
        } else {
          setLooseTabs(looseTabs.filter((tab) => tab.id !== activeId));
        }
        return;
      }

      return;
    }

    // Handle dropping onto Space section
    if (overId === "space-section" || overData?.type === "space") {
      // Handle folder being dropped
      if (activeData?.type === "folder") {
        // Folders can't be dropped into Space section directly
        // They stay as folders
        return;
      }

      // Find the dragged tab
      let draggedTab = looseTabs.find((tab) => tab.id === activeId);
      let sourceFolder: TabFolder | undefined;

      if (!draggedTab) {
        // Check if it's from a folder
        for (const folder of folders) {
          const tab = folder.tabs.find((t) => t.id === activeId);
          if (tab) {
            draggedTab = tab;
            sourceFolder = folder;
            break;
          }
        }
      }

      if (!draggedTab) return;

      // Remove from source and add to loose tabs
      if (sourceFolder) {
        // Moving from folder to loose tabs
        setFolders(
          folders.map((folder) =>
            folder.id === sourceFolder.id
              ? {
                  ...folder,
                  tabs: folder.tabs.filter((t) => t.id !== activeId),
                }
              : folder,
          ),
        );
        setLooseTabs([...looseTabs, { ...draggedTab, folderId: null }]);
      }
      // If already in loose tabs, no action needed
      return;
    }

    // Handle folder reordering and folder-to-folder drops
    if (activeData?.type === "folder") {
      const draggedFolder = folders.find((f) => f.id === activeId);
      if (!draggedFolder) return;

      // Dropping folder onto another folder (merge or reorder)
      if (overData?.type === "folder") {
        // For now, just reorder folders
        const oldIndex = folders.findIndex((f) => f.id === activeId);
        const newIndex = folders.findIndex((f) => f.id === overId);
        if (oldIndex !== -1 && newIndex !== -1 && oldIndex !== newIndex) {
          const newFolders = [...folders];
          const [removed] = newFolders.splice(oldIndex, 1);
          newFolders.splice(newIndex, 0, removed);
          setFolders(newFolders);
        }
        return;
      }

      // Dropping folder onto a tab (reorder folders)
      if (overData?.type === "tab") {
        // Just reorder folders, don't merge
        return;
      }

      return;
    }

    // Find the dragged tab from either loose tabs or folder tabs
    let draggedTab = looseTabs.find((tab) => tab.id === activeId);
    let sourceFolder: TabFolder | undefined;

    if (!draggedTab) {
      // Check if it's from a folder
      for (const folder of folders) {
        const tab = folder.tabs.find((t) => t.id === activeId);
        if (tab) {
          draggedTab = tab;
          sourceFolder = folder;
          break;
        }
      }
    }

    if (!draggedTab) return;

    const targetFolder = folders.find((folder) => folder.id === overId);

    // Dropping onto a folder
    if (targetFolder && overData?.type === "folder") {
      // Remove from source
      if (sourceFolder) {
        // Moving from folder to folder
        setFolders(
          folders.map((folder) => {
            if (folder.id === sourceFolder.id) {
              return {
                ...folder,
                tabs: folder.tabs.filter((t) => t.id !== activeId),
              };
            }
            if (folder.id === overId) {
              return {
                ...folder,
                tabs: [...folder.tabs, { ...draggedTab, folderId: overId }],
              };
            }
            return folder;
          }),
        );
      } else {
        // Moving from loose tabs to folder
        setLooseTabs(looseTabs.filter((tab) => tab.id !== activeId));
        setFolders(
          folders.map((folder) =>
            folder.id === overId
              ? {
                  ...folder,
                  tabs: [...folder.tabs, { ...draggedTab, folderId: overId }],
                }
              : folder,
          ),
        );
      }
    } else if (overData?.type === "tab") {
      // Dropping onto another tab - reorder or move between containers
      const targetTab = looseTabs.find((t) => t.id === overId);
      const targetTabFolder = folders.find((f) =>
        f.tabs.some((t) => t.id === overId),
      );

      if (targetTab) {
        // Target is in loose tabs
        if (sourceFolder) {
          // Moving from folder to loose tabs
          setFolders(
            folders.map((folder) =>
              folder.id === sourceFolder.id
                ? {
                    ...folder,
                    tabs: folder.tabs.filter((t) => t.id !== activeId),
                  }
                : folder,
            ),
          );
          const targetIndex = looseTabs.findIndex((t) => t.id === overId);
          const newTabs = [...looseTabs];
          newTabs.splice(targetIndex, 0, { ...draggedTab, folderId: null });
          setLooseTabs(newTabs);
        } else {
          // Reordering within loose tabs
          const oldIndex = looseTabs.findIndex((t) => t.id === activeId);
          const newIndex = looseTabs.findIndex((t) => t.id === overId);

          if (oldIndex !== -1 && newIndex !== -1 && oldIndex !== newIndex) {
            const newTabs = [...looseTabs];
            const [removed] = newTabs.splice(oldIndex, 1);
            newTabs.splice(newIndex, 0, removed);
            setLooseTabs(newTabs);
          }
        }
      } else if (targetTabFolder) {
        // Target is in a folder
        if (sourceFolder && sourceFolder.id === targetTabFolder.id) {
          // Reordering within same folder
          setFolders(
            folders.map((folder) => {
              if (folder.id === sourceFolder.id) {
                const oldIndex = folder.tabs.findIndex(
                  (t) => t.id === activeId,
                );
                const newIndex = folder.tabs.findIndex((t) => t.id === overId);
                if (
                  oldIndex !== -1 &&
                  newIndex !== -1 &&
                  oldIndex !== newIndex
                ) {
                  const newTabs = [...folder.tabs];
                  const [removed] = newTabs.splice(oldIndex, 1);
                  newTabs.splice(newIndex, 0, removed);
                  return { ...folder, tabs: newTabs };
                }
              }
              return folder;
            }),
          );
        } else {
          // Moving to different folder
          if (sourceFolder) {
            // From folder to folder
            setFolders(
              folders.map((folder) => {
                if (folder.id === sourceFolder.id) {
                  return {
                    ...folder,
                    tabs: folder.tabs.filter((t) => t.id !== activeId),
                  };
                }
                if (folder.id === targetTabFolder.id) {
                  const targetIndex = folder.tabs.findIndex(
                    (t) => t.id === overId,
                  );
                  const newTabs = [...folder.tabs];
                  newTabs.splice(targetIndex, 0, {
                    ...draggedTab,
                    folderId: folder.id,
                  });
                  return { ...folder, tabs: newTabs };
                }
                return folder;
              }),
            );
          } else {
            // From loose tabs to folder
            setLooseTabs(looseTabs.filter((tab) => tab.id !== activeId));
            setFolders(
              folders.map((folder) => {
                if (folder.id === targetTabFolder.id) {
                  const targetIndex = folder.tabs.findIndex(
                    (t) => t.id === overId,
                  );
                  const newTabs = [...folder.tabs];
                  newTabs.splice(targetIndex, 0, {
                    ...draggedTab,
                    folderId: folder.id,
                  });
                  return { ...folder, tabs: newTabs };
                }
                return folder;
              }),
            );
          }
        }
      }
    }
  }

  function handleCreateWorkspace(workspace: {
    name: string;
    icon: { type: "emoji" | "icon" | "dot"; value: string };
    color: string;
  }): void {
    const workspaceName =
      workspace.name.trim() || `Workspace ${workspaces.length + 1}`;
    const newWorkspace: Workspace = {
      id: Date.now().toString(),
      name: workspaceName,
      color: workspace.color,
      icon: workspace.icon,
    };
    setWorkspaces([...workspaces, newWorkspace]);
    setActiveWorkspace(newWorkspace.id);
  }

  function handleUpdateWorkspace(
    workspaceId: string,
    updates: {
      name?: string;
      icon?: { type: "emoji" | "icon" | "dot"; value: string };
    },
  ): void {
    setWorkspaces(
      workspaces.map((w) =>
        w.id === workspaceId
          ? {
              ...w,
              ...(updates.name && { name: updates.name }),
              ...(updates.icon && { icon: updates.icon }),
            }
          : w,
      ),
    );
  }

  function renderWorkspaceIcon(
    workspace: Workspace,
    isActive: boolean = false,
  ): React.ReactElement {
    if (workspace.icon.type === "emoji") {
      return (
        <div className="flex h-5 w-5 items-center justify-center">
          <span
            className={cn(
              "text-xs leading-none transition-all duration-300 ease-[cubic-bezier(0.165,0.84,0.44,1)]",
              isActive ? "" : "grayscale hover:grayscale-0",
            )}
          >
            {workspace.icon.value}
          </span>
        </div>
      );
    }
    if (workspace.icon.type === "icon") {
      const IconComponent = (LucideIcons as any)[workspace.icon.value];
      if (IconComponent) {
        return (
          <div className="flex h-5 w-5 items-center justify-center">
            <IconComponent
              className={cn(
                "h-3 w-3 transition-colors duration-300 ease-[cubic-bezier(0.165,0.84,0.44,1)]",
                isActive
                  ? "text-primary"
                  : "text-muted-foreground hover:text-accent-foreground",
              )}
            />
          </div>
        );
      }
    }
    return (
      <div className="flex h-5 w-5 items-center justify-center">
        <div
          className={cn(
            "h-2 w-2 rounded-full transition-all duration-300 ease-[cubic-bezier(0.165,0.84,0.44,1)]",
            isActive
              ? "bg-primary"
              : "bg-muted-foreground hover:bg-accent-foreground",
          )}
        />
      </div>
    );
  }

  function scrollWorkspaces(direction: "left" | "right"): void {
    if (direction === "left") {
      setWorkspaceScrollPosition((prev) => Math.max(0, prev - 1));
    } else {
      setWorkspaceScrollPosition((prev) =>
        Math.min(workspaces.length - MAX_VISIBLE_WORKSPACES, prev + 1),
      );
    }
  }

  function startScrolling(direction: "left" | "right"): void {
    setIsScrolling(true);
    scrollWorkspaces(direction);

    scrollIntervalRef.current = setInterval(() => {
      scrollWorkspaces(direction);
    }, 150);
  }

  function stopScrolling(): void {
    setIsScrolling(false);
    if (scrollIntervalRef.current) {
      clearInterval(scrollIntervalRef.current);
      scrollIntervalRef.current = null;
    }
  }

  React.useEffect(() => {
    return () => {
      if (scrollIntervalRef.current) {
        clearInterval(scrollIntervalRef.current);
      }
    };
  }, []);

  const visibleWorkspaces = workspaces.slice(
    workspaceScrollPosition,
    workspaceScrollPosition + MAX_VISIBLE_WORKSPACES,
  );
  const canScrollLeft = workspaceScrollPosition > 0;
  const canScrollRight =
    workspaceScrollPosition < workspaces.length - MAX_VISIBLE_WORKSPACES;

  function closeTab(tabId: string): void {
    setLooseTabs(looseTabs.filter((tab) => tab.id !== tabId));
    setFolders(
      folders.map((folder) => ({
        ...folder,
        tabs: folder.tabs.filter((tab) => tab.id !== tabId),
      })),
    );
  }

  function clearAllTabs(): void {
    setLooseTabs([]);
  }

  const activeWorkspaceFolders = folders.filter(
    (folder) => folder.workspaceId === activeWorkspace && !folder.parentId,
  );
  const activeWorkspaceTabs = looseTabs.filter(
    (tab) => tab.workspaceId === activeWorkspace && !tab.folderId,
  );

  // Draggable Tab Component
  function DraggableTab({ tab }: { tab: Tab }): React.ReactElement {
    const {
      attributes,
      listeners,
      setNodeRef,
      transform,
      transition,
      isDragging,
    } = useSortable({
      id: tab.id,
      data: { type: "tab", tab },
    });

    const style = {
      transform: CSS.Transform.toString(transform),
      transition,
      opacity: isDragging ? 0.5 : 1,
    };

    return (
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <div
            ref={setNodeRef}
            style={style}
            {...attributes}
            {...listeners}
            className={cn(
              "group/item relative flex h-9 max-w-[93.5%] cursor-grab items-center gap-2 rounded-md px-2 text-sm select-none active:cursor-grabbing transition-colors",
              activeTab === tab.id
                ? "bg-accent text-accent-foreground"
                : "text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground",
              isDragging && "opacity-50",
            )}
            onClick={() => setActiveTab(tab.id)}
          >
            <div className="bg-destructive h-4 w-4 shrink-0 rounded-sm" />
            <span className="min-w-0 flex-1 truncate pr-6 text-xs">
              {tab.title}
            </span>
            <button
              className="absolute right-2 flex h-3 w-3 shrink-0 cursor-pointer items-center justify-center opacity-0 transition-opacity group-hover/item:opacity-100"
              onClick={(e: any) => {
                e.stopPropagation();
                closeTab(tab.id);
              }}
            >
              <X className="h-3 w-3" />
            </button>
            {/* Drop position indicator */}
            {overId === tab.id && dropPosition === "before" && (
              <div className="absolute -top-1 left-0 right-0 z-[1000000000000000000000000] flex items-center">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary h-0.5 flex-1" />
              </div>
            )}
            {overId === tab.id && dropPosition === "after" && (
              <div className="absolute -bottom-1 left-0 right-0 z-[1000000000000000000000000] flex items-center">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary h-0.5 flex-1" />
              </div>
            )}
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent className="border-border bg-card w-56">
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              addNewTab();
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            New Tab Below
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              createFolder();
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            New Folder
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Reload Tab
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Mute Tab
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Remove from Essentials
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Change Icon...
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Duplicate Tab
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Bookmark Tab...
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Move Tab
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Share
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Open in New Container Tab
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Select All Tabs
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-muted-foreground/50 focus:bg-accent focus:text-muted-foreground/50"
          >
            Close Duplicate Tabs
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Close Multiple Tabs
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Reopen Closed Tab
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Replace Essential URL with Current
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Reset Essential Tab
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    );
  }

  // Droppable Folder Component
  function DroppableFolder({
    folder,
  }: {
    folder: TabFolder;
  }): React.ReactElement {
    const {
      attributes,
      listeners,
      setNodeRef,
      transform,
      transition,
      isDragging,
    } = useSortable({
      id: folder.id,
      data: { type: "folder", folder },
    });

    const style = {
      transform: CSS.Transform.toString(transform),
      transition,
      opacity: isDragging ? 0.5 : 1,
    };

    const isDropTarget = overId === folder.id && dropPosition === "inside";

    return (
      <ContextMenu modal={false}>
        <ContextMenuTrigger asChild>
          <div
            ref={setNodeRef}
            style={style}
            {...attributes}
            {...listeners}
            className={cn(
              "text-muted-foreground hover:bg-accent hover:text-accent-foreground group/item relative flex h-10 w-full max-w-[95%] cursor-grab items-center gap-2 rounded-md px-2 text-sm transition-colors active:cursor-grabbing",
              isDropTarget && "bg-accent text-accent-foreground",
              isDragging && "opacity-50",
            )}
            onClick={(_e: any) => {
              toggleFolder(folder.id);
            }}
            onContextMenu={(e: any) => {
              e.stopPropagation();
            }}
          >
            {folder.collapsed ? (
              <Folder className="text-primary h-4 w-4 shrink-0" />
            ) : (
              <FolderOpen className="text-primary h-4 w-4 shrink-0" />
            )}
            <span className="text-foreground min-w-0 flex-1 truncate pr-6 text-sm">
              {folder.name}
            </span>
            <button
              className="absolute right-2 flex h-4 w-4 shrink-0 cursor-pointer items-center justify-center opacity-0 transition-opacity group-hover/item:opacity-100"
              onClick={(e: any) => {
                e.stopPropagation();
                deleteFolder(folder.id);
              }}
            >
              <X className="h-3 w-3" />
            </button>
            {/* Drop position indicator */}
            {overId === folder.id && dropPosition === "before" && (
              <div className="absolute -top-1 left-0 right-0 z-100 flex items-center pointer-events-none">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary h-0.5 flex-1" />
              </div>
            )}
            {overId === folder.id && dropPosition === "after" && (
              <div className="absolute -bottom-1 left-0 right-0 z-100 flex items-center pointer-events-none">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary h-0.5 flex-1" />
              </div>
            )}
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent className="border-border bg-card w-56">
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              const newName = prompt("Enter new folder name:", folder.name);
              if (newName?.trim()) {
                setFolders(
                  folders.map((f) =>
                    f.id === folder.id ? { ...f, name: newName.trim() } : f,
                  ),
                );
              }
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Rename Folder
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Change Icon...
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              setFolders(
                folders.map((f) =>
                  f.id === folder.id ? { ...f, tabs: [] } : f,
                ),
              );
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Unload All Tabs
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              const subfolderCount = folders.filter(
                (f) => f.parentId === folder.id,
              ).length;
              const newSubfolder: TabFolder = {
                id: `folder-${Date.now()}`,
                name: `Subfolder ${subfolderCount + 1}`,
                collapsed: false,
                workspaceId: activeWorkspace,
                parentId: folder.id,
                tabs: [],
              };
              setFolders([...folders, newSubfolder]);
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            New Subfolder
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Change Space...
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Convert folder to Space
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              const folderTabs = folder.tabs.map((tab) => ({
                ...tab,
                folderId: null,
              }));
              setLooseTabs([...looseTabs, ...folderTabs]);
              deleteFolder(folder.id);
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Unpack Folder
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              deleteFolder(folder.id);
            }}
            className="text-destructive focus:bg-accent focus:text-destructive"
          >
            Delete Folder
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    );
  }

  // Draggable Tab in Folder Component
  function DraggableTabInFolder({ tab }: { tab: Tab }): React.ReactElement {
    const {
      attributes,
      listeners,
      setNodeRef,
      transform,
      transition,
      isDragging,
    } = useSortable({
      id: tab.id,
      data: { type: "tab", tab },
    });

    const style = {
      transform: CSS.Transform.toString(transform),
      transition,
      opacity: isDragging ? 0.5 : 1,
    };

    return (
      <div
        ref={setNodeRef}
        style={style}
        {...attributes}
        {...listeners}
        className={cn(
          "group/welcome text-muted-foreground hover:bg-accent hover:text-accent-foreground relative flex h-9 w-full max-w-[95%] cursor-grab items-center gap-2 rounded-md px-2 text-sm transition-colors active:cursor-grabbing",
          isDragging && "opacity-50",
        )}
        onClick={() => setActiveTab(tab.id)}
      >
        <div className="bg-primary h-4 w-4 shrink-0 rounded-sm" />
        <span className="text-foreground min-w-0 flex-1 truncate pr-6 text-xs">
          {tab.title}
        </span>
        <button
          className="absolute right-2 flex h-3 w-3 shrink-0 cursor-pointer items-center justify-center opacity-0 transition-opacity group-hover/welcome:opacity-100"
          onClick={(e: any) => {
            e.stopPropagation();
            closeTab(tab.id);
          }}
        >
          <X className="h-3 w-3" />
        </button>
        {/* Drop position indicator */}
        {overId === tab.id && dropPosition === "before" && (
          <div className="absolute -top-1 left-0 right-0 z-100 flex items-center pointer-events-none">
            <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
            <div className="bg-primary h-0.5 flex-1" />
          </div>
        )}
        {overId === tab.id && dropPosition === "after" && (
          <div className="absolute -bottom-1 left-0 right-0 z-100 flex items-center pointer-events-none">
            <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
            <div className="bg-primary h-0.5 flex-1" />
          </div>
        )}
      </div>
    );
  }

  // Draggable Logo Component
  function DraggableLogo({ logo }: { logo: SVGLogo }): React.ReactElement {
    const {
      attributes,
      listeners,
      setNodeRef,
      transform,
      transition,
      isDragging,
      isOver,
    } = useSortable({
      id: `logo-${logo.id}`,
      data: { type: "logo", logo },
    });

    const style = {
      transform: CSS.Transform.toString(transform),
      transition,
      opacity: isDragging ? 0.5 : 1,
    };

    const LogoComponent = logo.component;

    function removeLogo(): void {
      if (confirm(`Remove "${logo.title}" from quick access?`)) {
        setLogos(logos.filter((l) => l.id !== logo.id));
      }
    }

    return (
      <ContextMenu modal={false}>
        <ContextMenuTrigger asChild>
          <div
            ref={setNodeRef}
            style={style}
            {...attributes}
            {...listeners}
            className={cn(
              "bg-background/90 hover:bg-accent relative flex h-16 cursor-grab items-center justify-center rounded-md transition-colors active:cursor-grabbing",
              isDragging && "opacity-50",
              isOver && "ring-2 ring-primary",
            )}
            title={logo.title}
          >
            <LogoComponent className="h-6 w-6" />
            {/* Drop position indicator */}
            {overId === `logo-${logo.id}` && dropPosition === "before" && (
              <div className="absolute -left-1 top-0 bottom-0 z-100 flex flex-col items-center pointer-events-none">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary w-0.5 flex-1" />
              </div>
            )}
            {overId === `logo-${logo.id}` && dropPosition === "after" && (
              <div className="absolute -right-1 top-0 bottom-0 z-100 flex flex-col items-center pointer-events-none">
                <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                <div className="bg-primary w-0.5 flex-1" />
              </div>
            )}
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent className="border-border bg-card w-56">
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              const newTitle = prompt("Enter new title:", logo.title);
              if (newTitle?.trim()) {
                setLogos(
                  logos.map((l) =>
                    l.id === logo.id ? { ...l, title: newTitle.trim() } : l,
                  ),
                );
              }
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Rename
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              removeLogo();
            }}
            className="text-destructive focus:bg-accent focus:text-destructive"
          >
            <X className="mr-2 h-4 w-4" />
            Remove
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    );
  }

  // Droppable Logo Container Component
  function DroppableLogoContainer(): React.ReactElement {
    const { setNodeRef } = useSortable({
      id: "logo-container",
      data: { type: "logo-container" },
    });

    function _removeLogo(logoId: number): void {
      setLogos(logos.filter((l) => l.id !== logoId));
    }

    function _clearAllLogos(): void {
      if (confirm("Are you sure you want to clear all logos?")) {
        setLogos([]);
      }
    }

    function _resetLogos(): void {
      setLogos([
        { id: 1, title: "React", component: ReactLight },
        { id: 2, title: "Next.js", component: NextjsIconDark },
        { id: 3, title: "TypeScript", component: Typescript },
        { id: 4, title: "GitHub", component: GithubDark },
        { id: 5, title: "Vercel", component: Vercel },
        { id: 6, title: "Figma", component: Figma },
        { id: 7, title: "Notion", component: Notion },
        { id: 8, title: "Slack", component: Slack },
        { id: 9, title: "Discord", component: Discord },
        { id: 10, title: "YouTube", component: Youtube },
        { id: 11, title: "Supabase", component: Supabase },
        { id: 12, title: "Stripe", component: Stripe },
      ]);
    }

    return (
      <ContextMenu modal={false}>
        <ContextMenuTrigger asChild>
          <div
            ref={setNodeRef}
            className={cn(
              "shrink-0 p-2 transition-all duration-200 rounded-md",
              logoContainerHovered && "bg-primary/20 ring-2 ring-primary",
            )}
          >
            <div className="mx-auto grid w-[95%] grid-cols-4 gap-2">
              {isMounted ? (
                <SortableContext
                  items={logos.map((l) => `logo-${l.id}`)}
                  strategy={verticalListSortingStrategy}
                >
                  {logos.map((logo) => (
                    <DraggableLogo key={logo.id} logo={logo} />
                  ))}
                </SortableContext>
              ) : (
                logos.map((logo) => {
                  const LogoComponent = logo.component;
                  return (
                    <div
                      key={logo.id}
                      className="bg-background/90 hover:bg-accent flex h-16 items-center justify-center rounded-md transition-colors"
                      title={logo.title}
                    >
                      <LogoComponent className="h-6 w-6" />
                    </div>
                  );
                })
              )}
            </div>
          </div>
        </ContextMenuTrigger>
        <ContextMenuContent className="border-border bg-card w-56">
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              addNewTab();
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            New Tab Below
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              createFolder();
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            New Folder
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Reload Tab
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Mute Tab
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Remove from Essentials
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Change Icon...
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Duplicate Tab
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Bookmark Tab...
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Move Tab
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Share
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Open in New Container Tab
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => {
              e.preventDefault();
              // Select all tabs in active workspace
              const allTabIds = [
                ...activeWorkspaceTabs.map((t) => t.id),
                ...activeWorkspaceFolders.flatMap((f) =>
                  f.tabs.map((t) => t.id),
                ),
              ];
              console.log("Selected tabs:", allTabIds);
            }}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Select All Tabs
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-muted-foreground/50 focus:bg-accent focus:text-muted-foreground/50"
          >
            Close Duplicate Tabs
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Close Multiple Tabs
            <ChevronRight className="ml-auto h-4 w-4" />
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Reopen Closed Tab
          </ContextMenuItem>
          <div className="border-border my-1 border-t" />
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Replace Essential URL with Current
          </ContextMenuItem>
          <ContextMenuItem
            onSelect={(e: any) => e.preventDefault()}
            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
          >
            Reset Essential Tab
          </ContextMenuItem>
        </ContextMenuContent>
      </ContextMenu>
    );
  }

  // Droppable New Tab Button Component
  function DroppableNewTabButton(): React.ReactElement {
    const { setNodeRef, isOver } = useSortable({
      id: "new-tab-button",
      data: { type: "new-tab-button" },
    });

    const isDropTarget = overId === "new-tab-button" || isOver;

    return (
      <div
        ref={setNodeRef}
        className="shrink-0 px-2 pt-3 pb-3 overflow-x-hidden"
      >
        <div className="border-border/50 relative border-t">
          <button
            onClick={clearAllTabs}
            className="text-muted-foreground hover:text-accent-foreground absolute -top-2 right-0 cursor-pointer bg-card px-1 text-xs opacity-0 transition-opacity group-hover:opacity-100"
          >
            Clear
          </button>
        </div>
        <div className="pt-3 relative">
          <Button
            onClick={addNewTab}
            className={cn(
              "px-2 text-muted-foreground hover:bg-accent hover:text-accent-foreground h-10 w-full justify-start gap-2 bg-transparent",
              isDropTarget && "bg-primary/20",
            )}
            variant="ghost"
          >
            <Plus className="h-4 w-4 shrink-0" />
            <span className="min-w-0 flex-1 truncate text-left text-sm">
              New Tab
            </span>
          </Button>
          {/* Drop position indicator */}
          {overId === "new-tab-button" && dropPosition === "after" && (
            <div className="absolute -bottom-1 left-0 right-0 z-100 flex items-center pointer-events-none">
              <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
              <div className="bg-primary h-0.5 flex-1" />
            </div>
          )}
        </div>
      </div>
    );
  }
  function DroppableSpace(): React.ReactElement {
    const [dropdownOpen, setDropdownOpen] = React.useState(false);
    const { setNodeRef, isOver } = useSortable({
      id: "space-section",
      data: { type: "space" },
    });

    const isDropTarget = overId === "space-section" || isOver;

    return (
      <div
        ref={setNodeRef}
        className="group/spacearea shrink-0 px-2"
        onMouseEnter={() => setIsSpaceAreaHovered(true)}
        onMouseLeave={() => setIsSpaceAreaHovered(false)}
      >
        <ContextMenu modal={false}>
          <ContextMenuTrigger asChild>
            <div
              className={cn(
                "hover:bg-accent relative flex h-10 w-full cursor-pointer items-center rounded-md px-2 transition-colors",
                isDropTarget && "bg-accent ring-2 ring-primary",
              )}
            >
              <AnimatePresence>
                {isSpaceAreaHovered && (
                  <motion.div
                    initial={{ width: 0, opacity: 0 }}
                    animate={{
                      width: "auto",
                      opacity: 1,
                    }}
                    exit={{ width: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="shrink-0 overflow-hidden"
                    onClick={() => setSpaceCollapsed(!spaceCollapsed)}
                  >
                    <motion.div
                      animate={{ rotate: spaceCollapsed ? 0 : 180 }}
                      transition={{ duration: 0.2 }}
                    >
                      <ChevronDown className="text-muted-foreground h-4 w-4" />
                    </motion.div>
                  </motion.div>
                )}
              </AnimatePresence>
              <motion.span
                className="text-muted-foreground min-w-0 flex-1 truncate text-left text-sm font-medium"
                animate={{ marginLeft: isSpaceAreaHovered ? 8 : 0 }}
                transition={{ duration: 0.2 }}
                onClick={() => setSpaceCollapsed(!spaceCollapsed)}
              >
                {workspaces.find((w) => w.id === activeWorkspace)?.name ||
                  "Space"}
              </motion.span>
              <AnimatePresence>
                {(isSpaceAreaHovered || dropdownOpen) && (
                  <motion.div
                    initial={{ width: 0, opacity: 0 }}
                    animate={{ width: "auto", opacity: 1 }}
                    exit={{ width: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="shrink-0 overflow-hidden"
                  >
                    <DropdownMenu
                      modal={false}
                      open={dropdownOpen}
                      onOpenChange={setDropdownOpen}
                    >
                      <DropdownMenuTrigger asChild>
                        <button
                          className="hover:bg-accent rounded p-0.5"
                          onClick={(e: any) => {
                            e.stopPropagation();
                          }}
                          onContextMenu={(e: any) => {
                            e.stopPropagation();
                          }}
                        >
                          <MoreHorizontal className="text-muted-foreground h-4 w-4" />
                        </button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent
                        side="right"
                        className="border-border bg-card w-56"
                      >
                        <DropdownMenuItem
                          onSelect={(e: any) => {
                            e.preventDefault();
                            const currentWorkspace = workspaces.find(
                              (w) => w.id === activeWorkspace,
                            );
                            const newName = prompt(
                              "Enter new workspace name:",
                              currentWorkspace?.name || "",
                            );
                            if (newName?.trim()) {
                              setWorkspaces(
                                workspaces.map((w) =>
                                  w.id === activeWorkspace
                                    ? { ...w, name: newName.trim() }
                                    : w,
                                ),
                              );
                            }
                          }}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Change Name
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onSelect={(e: any) => {
                            e.preventDefault();
                            const currentWorkspace = workspaces.find(
                              (w) => w.id === activeWorkspace,
                            );
                            if (currentWorkspace) {
                              setWorkspaceEditMode({
                                workspaceId: currentWorkspace.id,
                                currentName: currentWorkspace.name,
                                currentIcon: currentWorkspace.icon,
                              });
                              setWorkspaceDialogOpen(true);
                            }
                          }}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Change Icon
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onSelect={(e: any) => e.preventDefault()}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Edit Theme
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onSelect={(e: any) => e.preventDefault()}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Set Profile
                          <ChevronRight className="ml-auto h-4 w-4" />
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onSelect={(e: any) => {
                            e.preventDefault();
                            // Unload all tabs and folders in this workspace
                            setFolders(
                              folders.filter(
                                (f) => f.workspaceId !== activeWorkspace,
                              ),
                            );
                            setLooseTabs(
                              looseTabs.filter(
                                (t) => t.workspaceId !== activeWorkspace,
                              ),
                            );
                          }}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Unload Space
                        </DropdownMenuItem>
                        <div className="border-border my-1 border-t" />
                        <DropdownMenuItem
                          onSelect={(e: any) => e.preventDefault()}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                            ✓
                          </div>
                          <span className="max-w-[75%] truncate">
                            {workspaces.find((w) => w.id === activeWorkspace)
                              ?.name || "Space"}
                          </span>
                        </DropdownMenuItem>
                        {workspaces
                          .filter((w) => w.id !== activeWorkspace)
                          .map((w) => (
                            <DropdownMenuItem
                              key={w.id}
                              onSelect={(e: any) => {
                                e.preventDefault();
                                setActiveWorkspace(w.id);
                              }}
                              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                            >
                              <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                                {renderWorkspaceIcon(w, false)}
                              </div>
                              <span className="max-w-[75%] truncate">
                                {w.name}
                              </span>
                            </DropdownMenuItem>
                          ))}
                        <div className="border-border my-1 border-t" />
                        <DropdownMenuItem
                          onSelect={(e: any) => e.preventDefault()}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Reorder Spaces
                        </DropdownMenuItem>
                        <div className="border-border my-1 border-t" />
                        <DropdownMenuItem
                          onSelect={(e: any) => {
                            e.preventDefault();
                            createFolder();
                          }}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          <Folder className="mr-2 h-4 w-4" />
                          Create Folder
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onSelect={(e: any) => {
                            e.preventDefault();
                            setWorkspaceEditMode(undefined);
                            setWorkspaceDialogOpen(true);
                          }}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Create Space
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onSelect={(e: any) => {
                            e.preventDefault();
                            if (workspaces.length <= 1) {
                              alert("Cannot delete the last workspace!");
                              return;
                            }
                            const currentWorkspace = workspaces.find(
                              (w) => w.id === activeWorkspace,
                            );
                            if (
                              confirm(
                                `Are you sure you want to delete "${currentWorkspace?.name}"? All tabs and folders will be removed.`,
                              )
                            ) {
                              // Remove all tabs and folders in this workspace
                              setFolders(
                                folders.filter(
                                  (f) => f.workspaceId !== activeWorkspace,
                                ),
                              );
                              setLooseTabs(
                                looseTabs.filter(
                                  (t) => t.workspaceId !== activeWorkspace,
                                ),
                              );
                              // Remove the workspace
                              setWorkspaces(
                                workspaces.filter(
                                  (w) => w.id !== activeWorkspace,
                                ),
                              );
                              // Switch to the first remaining workspace
                              const remainingWorkspaces = workspaces.filter(
                                (w) => w.id !== activeWorkspace,
                              );
                              if (remainingWorkspaces.length > 0) {
                                setActiveWorkspace(remainingWorkspaces[0].id);
                              }
                            }
                          }}
                          className="text-destructive focus:bg-accent focus:text-destructive"
                        >
                          Delete Space
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </motion.div>
                )}
              </AnimatePresence>
              {/* Drop position indicator for Space */}
              {overId === "space-section" && dropPosition === "before" && (
                <div className="absolute -top-1 left-0 right-0 z-100 flex items-center pointer-events-none">
                  <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                  <div className="bg-primary h-0.5 flex-1" />
                </div>
              )}
              {overId === "space-section" && dropPosition === "after" && (
                <div className="absolute -bottom-1 left-0 right-0 z-100 flex items-center pointer-events-none">
                  <div className="bg-primary h-2 w-2 shrink-0 rounded-full" />
                  <div className="bg-primary h-0.5 flex-1" />
                </div>
              )}
            </div>
          </ContextMenuTrigger>
          <ContextMenuContent className="border-border bg-card w-56">
            <ContextMenuItem
              onSelect={(e: any) => {
                e.preventDefault();
                createFolder();
              }}
              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
            >
              <Folder className="mr-2 h-4 w-4" />
              Create Folder
            </ContextMenuItem>
            <ContextMenuItem
              onSelect={(e: any) => {
                e.preventDefault();
                addNewTab();
              }}
              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
            >
              <Plus className="mr-2 h-4 w-4" />
              New Tab
            </ContextMenuItem>
            <div className="border-border my-1 border-t" />
            <ContextMenuItem
              onSelect={(e: any) => e.preventDefault()}
              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
            >
              Collapse All Folders
            </ContextMenuItem>
            <ContextMenuItem
              onSelect={(e: any) => e.preventDefault()}
              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
            >
              Expand All Folders
            </ContextMenuItem>
          </ContextMenuContent>
        </ContextMenu>
      </div>
    );
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragStart={handleDragStart}
      onDragOver={handleDragOver}
      onDragEnd={handleDragEnd}
    >
      <div className="bg-background flex h-screen w-screen overflow-hidden">
        <div
          className={cn(
            "group bg-card flex h-full shrink-0 flex-col transition-all duration-200 select-none overflow-x-hidden",
            sidebarExpanded ? "w-[360px]" : "w-14",
          )}
        >
          <div className="grid h-11 shrink-0 grid-cols-2 gap-px">
            {sidebarExpanded ? (
              <>
                <div className="bg-card flex items-center gap-1 px-2">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                    onClick={() => setSidebarExpanded(false)}
                  >
                    <PanelLeft className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                  >
                    <Cog className="h-4 w-4" />
                  </Button>
                </div>
                <div className="bg-card flex items-center justify-end gap-1 px-2">
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                  >
                    <ShieldAlert className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                  >
                    <ShieldBan className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </div>
              </>
            ) : (
              <div className="bg-card col-span-2 flex items-center justify-center">
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                  onClick={() => setSidebarExpanded(true)}
                >
                  <PanelLeft className="h-4 w-4" />
                </Button>
              </div>
            )}
          </div>

          {!sidebarExpanded && (
            <div className="flex flex-1 flex-col items-center py-2">
              <div className="relative mt-auto flex flex-col items-center pb-3">
                {/* Top scroll arrow */}
                {canScrollLeft && (
                  <motion.button
                    whileTap={{ scale: 0.9 }}
                    onMouseDown={() => startScrolling("left")}
                    onMouseUp={stopScrolling}
                    onMouseLeave={stopScrolling}
                    onTouchStart={() => startScrolling("left")}
                    onTouchEnd={stopScrolling}
                    className="bg-background text-primary z-50 mb-1 flex h-8 w-8 items-center justify-center rounded-full border"
                    aria-label="Scroll workspaces up"
                  >
                    <ChevronLeft className="h-4 w-4 rotate-90" />
                  </motion.button>
                )}

                {/* Workspace dots - scrollable with reduced gap */}
                <div className="flex max-h-[240px] flex-col items-center overflow-hidden">
                  {visibleWorkspaces.map((workspace) => (
                    <ContextMenu key={workspace.id}>
                      <ContextMenuTrigger asChild>
                        <motion.button
                          layout
                          initial={{ opacity: 0, scale: 0.8 }}
                          animate={{ opacity: 1, scale: 1 }}
                          exit={{ opacity: 0, scale: 0.8 }}
                          transition={{
                            layout: {
                              type: "spring",
                              stiffness: 400,
                              damping: 30,
                            },
                            opacity: { duration: 0.2 },
                            scale: { duration: 0.2 },
                          }}
                          onClick={() => setActiveWorkspace(workspace.id)}
                          className={cn(
                            "group flex h-6 w-6 shrink-0 items-center justify-center rounded-md",
                            activeWorkspace === workspace.id && "scale-110",
                          )}
                          title={workspace.name}
                          aria-label={`Switch to ${workspace.name} workspace`}
                          whileHover={{ scale: 1.25 }}
                          whileTap={{ scale: 0.95 }}
                          drag
                          dragConstraints={{
                            left: 0,
                            right: 0,
                            top: 0,
                            bottom: 0,
                          }}
                          dragElastic={0.2}
                        >
                          {renderWorkspaceIcon(
                            workspace,
                            activeWorkspace === workspace.id,
                          )}
                        </motion.button>
                      </ContextMenuTrigger>
                      <ContextMenuContent className="border-border bg-card w-56">
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Change Name
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Change Icon
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Edit Theme
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Set Profile
                          <ChevronRight className="ml-auto h-4 w-4" />
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Unload Space
                        </ContextMenuItem>
                        <div className="border-border my-1 border-t" />
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                            ✓
                          </div>
                          <span className="max-w-[75%] truncate">
                            {workspace.name}
                          </span>
                        </ContextMenuItem>
                        {workspaces
                          .filter((w) => w.id !== workspace.id)
                          .map((w) => (
                            <ContextMenuItem
                              key={w.id}
                              onClick={() => setActiveWorkspace(w.id)}
                              className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                            >
                              <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                                {renderWorkspaceIcon(w, false)}
                              </div>
                              <span className="max-w-[75%] truncate">
                                {w.name}
                              </span>
                            </ContextMenuItem>
                          ))}
                        <div className="border-border my-1 border-t" />
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Reorder Spaces
                        </ContextMenuItem>
                        <div className="border-border my-1 border-t" />
                        <ContextMenuItem
                          onClick={() => {
                            setWorkspaceDialogOpen(true);
                          }}
                          className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                        >
                          Create Space
                        </ContextMenuItem>
                        <ContextMenuItem className="text-destructive focus:bg-accent focus:text-destructive">
                          Delete Space
                        </ContextMenuItem>
                      </ContextMenuContent>
                    </ContextMenu>
                  ))}
                </div>

                {/* Bottom scroll arrow */}
                {canScrollRight && (
                  <motion.button
                    whileTap={{ scale: 0.9 }}
                    onMouseDown={() => startScrolling("right")}
                    onMouseUp={stopScrolling}
                    onMouseLeave={stopScrolling}
                    onTouchStart={() => startScrolling("right")}
                    onTouchEnd={stopScrolling}
                    className="bg-background text-primary z-50 mt-1 flex h-8 w-8 items-center justify-center rounded-full border"
                    aria-label="Scroll workspaces down"
                  >
                    <ChevronRight className="h-4 w-4 rotate-90" />
                  </motion.button>
                )}
              </div>

              <div className="border-border flex flex-col items-center gap-2 border-t pt-3">
                <ContextMenu>
                  <ContextMenuTrigger asChild>
                    <motion.div
                      whileHover={{ scale: 1.05 }}
                      whileTap={{ scale: 0.95 }}
                      drag
                      dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
                      dragElastic={0.1}
                    >
                      <Popover
                        open={downloadsOpen}
                        onOpenChange={setDownloadsOpen}
                      >
                        <PopoverTrigger asChild>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                          >
                            <CircleDashed className="h-4 w-4" />
                          </Button>
                        </PopoverTrigger>
                        <PopoverContent
                          side="right"
                          className="border-border bg-card w-80 p-4"
                        >
                          <div className="space-y-4">
                            <p className="text-muted-foreground text-sm">
                              No downloads for this session.
                            </p>
                            <div className="border-border border-t pt-4">
                              <button className="text-accent-foreground hover:text-accent-foreground text-sm underline">
                                Show all downloads
                              </button>
                            </div>
                          </div>
                        </PopoverContent>
                      </Popover>
                    </motion.div>
                  </ContextMenuTrigger>
                  <ContextMenuContent className="border-border bg-card">
                    <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                      Open Downloads
                    </ContextMenuItem>
                    <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                      Clear Downloads
                    </ContextMenuItem>
                  </ContextMenuContent>
                </ContextMenu>

                <motion.div
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  drag
                  dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
                  dragElastic={0.1}
                  transition={{
                    type: "spring",
                    stiffness: 400,
                    damping: 25,
                    mass: 0.5,
                  }}
                >
                  <DropdownMenu
                    open={plusMenuOpen}
                    onOpenChange={setPlusMenuOpen}
                  >
                    <DropdownMenuTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                        onClick={(e: any) => {
                          e.stopPropagation();
                          handleCreateWorkspace({
                            name: "",
                            icon: { type: "dot", value: "" },
                            color: COLORS[workspaces.length % COLORS.length],
                          });
                        }}
                        onContextMenu={(e: any) => {
                          e.preventDefault();
                          e.stopPropagation();
                          setPlusMenuOpen(true);
                        }}
                      >
                        <Plus className="h-4 w-4" />
                      </Button>
                    </DropdownMenuTrigger>
                    <DropdownMenuContent
                      side="right"
                      align="end"
                      sideOffset={5}
                      className="border-border bg-card w-56"
                    >
                      <DropdownMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                        <Folder className="mr-2 h-4 w-4" />
                        Live Folder
                        <ChevronRight className="ml-auto h-4 w-4" />
                      </DropdownMenuItem>
                      <div className="border-border my-1 border-t" />
                      <DropdownMenuItem
                        onClick={() => {
                          setWorkspaceDialogOpen(true);
                          setPlusMenuOpen(false);
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        <Copy className="mr-2 h-4 w-4" />
                        Create Space
                      </DropdownMenuItem>
                      <DropdownMenuItem
                        onClick={() => {
                          createFolder();
                          setPlusMenuOpen(false);
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        <Folder className="mr-2 h-4 w-4" />
                        Create Folder
                      </DropdownMenuItem>
                      <div className="border-border my-1 border-t" />
                      <DropdownMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                        <Columns2 className="mr-2 h-4 w-4" />
                        New Split
                      </DropdownMenuItem>
                      <div className="border-border my-1 border-t" />
                      <DropdownMenuItem
                        onClick={() => {
                          addNewTab();
                          setPlusMenuOpen(false);
                        }}
                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                      >
                        <Plus className="mr-2 h-4 w-4" />
                        New Tab
                      </DropdownMenuItem>
                    </DropdownMenuContent>
                  </DropdownMenu>
                </motion.div>
              </div>
            </div>
          )}

          <AnimatePresence>
            {sidebarExpanded && (
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.15 }}
                className="flex flex-1 flex-col overflow-hidden overflow-x-hidden"
              >
                <div className="shrink-0 px-2 pt-1 pb-3">
                  <button
                    onClick={() => setCommandOpen(true)}
                    className="bg-accent hover:bg-muted relative flex w-full items-center gap-2 rounded-lg px-3 py-2.5 transition-colors"
                  >
                    <Search className="text-muted-foreground h-4 w-4 shrink-0" />
                    <span className="text-muted-foreground flex-1 text-left text-sm">
                      Search or enter address...
                    </span>
                    <div className="flex items-center gap-1">
                      <Link className="text-muted-foreground h-4 w-4" />
                      <Grid3x3 className="text-muted-foreground h-4 w-4" />
                    </div>
                  </button>
                </div>

                {isMounted ? (
                  <DroppableLogoContainer />
                ) : (
                  <div className="shrink-0 p-px">
                    <div className="mx-auto grid w-[95%] grid-cols-4 gap-2">
                      {logos.map((logo) => {
                        const LogoComponent = logo.component;
                        return (
                          <div
                            key={logo.id}
                            className="bg-background/90 hover:bg-accent flex h-16 items-center justify-center rounded-md transition-colors"
                            title={logo.title}
                          >
                            <LogoComponent className="h-6 w-6" />
                          </div>
                        );
                      })}
                    </div>
                  </div>
                )}

                {isMounted ? (
                  <DroppableSpace />
                ) : (
                  <div
                    className="group/spacearea shrink-0 px-2 pt-3 pb-2"
                    onMouseEnter={() => setIsSpaceAreaHovered(true)}
                    onMouseLeave={() => setIsSpaceAreaHovered(false)}
                  >
                    <div className="hover:bg-accent relative flex h-10 w-full cursor-pointer items-center rounded-md px-2 transition-colors">
                      <span className="text-muted-foreground min-w-0 flex-1 truncate text-left text-sm font-medium">
                        {workspaces.find((w) => w.id === activeWorkspace)
                          ?.name || "Space"}
                      </span>
                    </div>
                  </div>
                )}

                <AnimatePresence>
                  {!spaceCollapsed && (
                    <motion.div
                      initial={{ height: 0, opacity: 0 }}
                      animate={{ height: "auto", opacity: 1 }}
                      exit={{ height: 0, opacity: 0 }}
                      transition={{ duration: 0.2 }}
                      className="overflow-hidden"
                      onMouseEnter={() => setIsSpaceAreaHovered(true)}
                      onMouseLeave={() => setIsSpaceAreaHovered(false)}
                    >
                      <div className="shrink-0 space-y-1 px-2 pb-3">
                        {isMounted ? (
                          <SortableContext
                            items={activeWorkspaceFolders.map((f) => f.id)}
                            strategy={verticalListSortingStrategy}
                          >
                            {activeWorkspaceFolders.map((folder) => (
                              <div key={folder.id}>
                                <DroppableFolder folder={folder} />
                                <AnimatePresence>
                                  {!folder.collapsed &&
                                    folder.tabs.length > 0 && (
                                      <motion.div
                                        initial={{ height: 0, opacity: 0 }}
                                        animate={{ height: "auto", opacity: 1 }}
                                        exit={{ height: 0, opacity: 0 }}
                                        transition={{ duration: 0.2 }}
                                        className="ml-5 space-y-0.5"
                                      >
                                        <SortableContext
                                          items={folder.tabs.map((t) => t.id)}
                                          strategy={verticalListSortingStrategy}
                                        >
                                          {folder.tabs.map((tab) => (
                                            <DraggableTabInFolder
                                              key={tab.id}
                                              tab={tab}
                                            />
                                          ))}
                                        </SortableContext>
                                      </motion.div>
                                    )}
                                </AnimatePresence>
                              </div>
                            ))}
                          </SortableContext>
                        ) : (
                          activeWorkspaceFolders.map((folder) => (
                            <div key={folder.id}>
                              <div className="text-muted-foreground hover:bg-accent hover:text-accent-foreground group/item relative flex h-10 w-full cursor-pointer items-center gap-2 rounded-md px-2 text-sm transition-colors overflow-hidden">
                                <Folder className="text-primary h-4 w-4 shrink-0" />
                                <span className="text-foreground min-w-0 flex-1 truncate pr-5 text-left text-sm">
                                  {folder.name}
                                </span>
                              </div>
                            </div>
                          ))
                        )}
                      </div>
                    </motion.div>
                  )}
                </AnimatePresence>

                {isMounted ? (
                  <DroppableNewTabButton />
                ) : (
                  <div className="shrink-0 px-2 pt-3 pb-3 overflow-x-hidden">
                    <div className="border-border/50 relative border-t">
                      <button
                        onClick={clearAllTabs}
                        className="text-muted-foreground hover:text-accent-foreground absolute -top-2 right-0 cursor-pointer bg-card px-1 text-xs opacity-0 transition-opacity group-hover:opacity-100"
                      >
                        Clear
                      </button>
                    </div>
                    <div className="pt-3">
                      <Button
                        onClick={addNewTab}
                        className="px-2 text-muted-foreground hover:bg-accent hover:text-accent-foreground h-10 w-full justify-start gap-2 bg-transparent overflow-hidden"
                        variant="ghost"
                      >
                        <Plus className="h-4 w-4 shrink-0" />
                        <span className="min-w-0 flex-1 truncate text-sm">
                          New Tab
                        </span>
                      </Button>
                    </div>
                  </div>
                )}

                <ScrollArea className="flex-1 overflow-hidden overflow-x-hidden px-2">
                  <div className="w-full max-w-full space-y-0.5 pb-4">
                    {isMounted ? (
                      <SortableContext
                        items={activeWorkspaceTabs.map((t) => t.id)}
                        strategy={verticalListSortingStrategy}
                      >
                        {activeWorkspaceTabs.map((tab) => (
                          <DraggableTab key={tab.id} tab={tab} />
                        ))}
                      </SortableContext>
                    ) : (
                      activeWorkspaceTabs.map((tab) => (
                        <div
                          key={tab.id}
                          className="group/item relative flex h-9 w-full max-w-[95%] cursor-pointer items-center gap-2 rounded-md px-2 text-sm select-none text-muted-foreground hover:bg-accent/50 hover:text-accent-foreground"
                        >
                          <div className="bg-destructive h-4 w-4 shrink-0 rounded-sm" />
                          <span className="min-w-0 flex-1 truncate pr-6 text-xs">
                            {tab.title}
                          </span>
                        </div>
                      ))
                    )}
                  </div>
                </ScrollArea>

                {/* Media Player */}
                {mediaPlaying && (
                  <motion.div
                    initial={{ opacity: 0, y: 20 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="border-border bg-accent/30 mx-auto w-[95%] shrink-0 rounded-md border-t p-3"
                  >
                    <div className="space-y-2">
                      <div className="flex items-start justify-between gap-2">
                        <div className="min-w-0 flex-1">
                          <p className="text-secondary-foreground truncate text-xs font-medium">
                            Every Level Of Intelligence Explained in 5 Minutes
                          </p>
                          <p className="text-muted-foreground truncate text-xs">
                            Nick Explains
                          </p>
                        </div>
                        <button
                          onClick={() => setMediaPlaying(false)}
                          className="text-muted-foreground hover:text-accent-foreground shrink-0"
                        >
                          <X className="h-3 w-3" />
                        </button>
                      </div>

                      <div className="text-muted-foreground flex items-center gap-1 text-xs">
                        <span>1:40</span>
                        <div className="bg-muted relative h-1 flex-1 overflow-hidden rounded-full">
                          <div
                            className="bg-primary h-full"
                            style={{ width: `${mediaProgress}%` }}
                          />
                        </div>
                        <span>5:07</span>
                      </div>

                      <div className="flex items-center justify-between">
                        <div className="bg-destructive flex h-6 w-6 items-center justify-center rounded">
                          <Youtube className="text-destructive-foreground h-3 w-3" />
                        </div>

                        <div className="flex items-center gap-1">
                          <button className="text-muted-foreground hover:text-accent-foreground">
                            <SkipBack className="h-4 w-4" />
                          </button>
                          <button className="text-accent-foreground hover:text-accent-foreground">
                            <Play className="h-4 w-4" />
                          </button>
                          <button className="text-muted-foreground hover:text-accent-foreground">
                            <SkipForward className="h-4 w-4" />
                          </button>
                        </div>

                        <button className="text-muted-foreground hover:text-accent-foreground">
                          <Volume2 className="h-4 w-4" />
                        </button>
                      </div>
                    </div>
                  </motion.div>
                )}

                <div className="border-border shrink-0">
                  <motion.div
                    className="flex items-center justify-between p-2"
                    initial={{ y: 20, opacity: 0 }}
                    animate={{ y: 0, opacity: 1 }}
                    transition={{ delay: 0.1 }}
                  >
                    <ContextMenu>
                      <ContextMenuTrigger asChild>
                        <motion.div
                          whileHover={{ scale: 1.05 }}
                          whileTap={{ scale: 0.95 }}
                          drag
                          dragConstraints={{
                            left: 0,
                            right: 0,
                            top: 0,
                            bottom: 0,
                          }}
                          dragElastic={0.1}
                          transition={{
                            type: "spring",
                            stiffness: 400,
                            damping: 25,
                            mass: 0.5,
                          }}
                        >
                          <Popover
                            open={downloadsOpen}
                            onOpenChange={setDownloadsOpen}
                          >
                            <PopoverTrigger asChild>
                              <Button
                                variant="ghost"
                                size="icon"
                                className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                              >
                                <CircleDashed className="h-4 w-4" />
                              </Button>
                            </PopoverTrigger>
                            <PopoverContent
                              side="top"
                              align="start"
                              className="border-border bg-card w-80 p-4"
                            >
                              <div className="space-y-4">
                                <p className="text-muted-foreground text-sm">
                                  No downloads for this session.
                                </p>
                                <div className="border-border border-t pt-4">
                                  <button className="text-accent-foreground hover:text-accent-foreground text-sm underline">
                                    Show all downloads
                                  </button>
                                </div>
                              </div>
                            </PopoverContent>
                          </Popover>
                        </motion.div>
                      </ContextMenuTrigger>
                      <ContextMenuContent className="border-border bg-card">
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Open Downloads
                        </ContextMenuItem>
                        <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                          Clear Downloads
                        </ContextMenuItem>
                      </ContextMenuContent>
                    </ContextMenu>

                    <div className="relative flex w-38 shrink-0 items-center justify-center overflow-hidden">
                      {/* Left Arrow - Fixed position, always visible */}
                      <motion.button
                        whileTap={canScrollLeft ? { scale: 0.9 } : {}}
                        onMouseDown={() =>
                          canScrollLeft && startScrolling("left")
                        }
                        onMouseUp={stopScrolling}
                        onMouseLeave={stopScrolling}
                        onTouchStart={() =>
                          canScrollLeft && startScrolling("left")
                        }
                        onTouchEnd={stopScrolling}
                        className={cn(
                          "bg-background text-primary absolute left-0 z-10 flex h-6 w-6 items-center justify-center rounded-full border transition-all duration-200",
                          canScrollLeft
                            ? "text-muted-foreground hover:text-accent-foreground cursor-pointer"
                            : "hidden!",
                        )}
                        aria-label="Scroll workspaces left"
                        aria-disabled={!canScrollLeft}
                      >
                        <ChevronLeft className="h-3 w-3" />
                      </motion.button>

                      {/* Workspace dots container - Fixed width with overflow hidden */}
                      <div className="flex h-6 items-center justify-center px-2">
                        <div className="flex items-center gap-1">
                          <AnimatePresence mode="popLayout">
                            {visibleWorkspaces.map((workspace) => (
                              <ContextMenu key={workspace.id}>
                                <ContextMenuTrigger asChild>
                                  <motion.button
                                    layout
                                    initial={{ opacity: 0, scale: 0.8 }}
                                    animate={{ opacity: 1, scale: 1 }}
                                    exit={{ opacity: 0, scale: 0.8 }}
                                    transition={{
                                      layout: {
                                        type: "spring",
                                        stiffness: 400,
                                        damping: 30,
                                      },
                                      opacity: { duration: 0.2 },
                                      scale: { duration: 0.2 },
                                    }}
                                    onClick={() =>
                                      setActiveWorkspace(workspace.id)
                                    }
                                    className={cn(
                                      "group flex h-4 w-4 shrink-0 items-center justify-center rounded-md",
                                      activeWorkspace === workspace.id &&
                                        "scale-110",
                                    )}
                                    title={workspace.name}
                                    aria-label={`Switch to ${workspace.name} workspace`}
                                    whileHover={{ scale: 1.25 }}
                                    whileTap={{ scale: 0.95 }}
                                    drag
                                    dragConstraints={{
                                      left: 0,
                                      right: 0,
                                      top: 0,
                                      bottom: 0,
                                    }}
                                    dragElastic={0.2}
                                  >
                                    {renderWorkspaceIcon(
                                      workspace,
                                      activeWorkspace === workspace.id,
                                    )}
                                  </motion.button>
                                </ContextMenuTrigger>
                                <ContextMenuContent className="border-border bg-card w-56">
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    Change Name
                                  </ContextMenuItem>
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    Change Icon
                                  </ContextMenuItem>
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    Edit Theme
                                  </ContextMenuItem>
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    Set Profile
                                    <ChevronRight className="ml-auto h-4 w-4" />
                                  </ContextMenuItem>
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    Unload Space
                                  </ContextMenuItem>
                                  <div className="border-border my-1 border-t" />
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                                      ✓
                                    </div>
                                    <span className="max-w-[75%] truncate">
                                      {workspace.name}
                                    </span>
                                  </ContextMenuItem>
                                  {workspaces
                                    .filter((w) => w.id !== workspace.id)
                                    .map((w) => (
                                      <ContextMenuItem
                                        key={w.id}
                                        onClick={() => setActiveWorkspace(w.id)}
                                        className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                                      >
                                        <div className="mr-2 h-4 w-4 flex items-center justify-center shrink-0">
                                          {renderWorkspaceIcon(w, false)}
                                        </div>
                                        <span className="max-w-[75%] truncate">
                                          {w.name}
                                        </span>
                                      </ContextMenuItem>
                                    ))}
                                  <div className="border-border my-1 border-t" />
                                  <ContextMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                                    Reorder Spaces
                                  </ContextMenuItem>
                                  <div className="border-border my-1 border-t" />
                                  <ContextMenuItem
                                    onClick={() => {
                                      setWorkspaceDialogOpen(true);
                                    }}
                                    className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                                  >
                                    Create Space
                                  </ContextMenuItem>
                                  <ContextMenuItem className="text-destructive focus:bg-accent focus:text-destructive">
                                    Delete Space
                                  </ContextMenuItem>
                                </ContextMenuContent>
                              </ContextMenu>
                            ))}
                          </AnimatePresence>
                        </div>
                      </div>

                      {/* Right Arrow - Fixed position, always visible */}
                      <motion.button
                        whileTap={canScrollRight ? { scale: 0.9 } : {}}
                        onMouseDown={() =>
                          canScrollRight && startScrolling("right")
                        }
                        onMouseUp={stopScrolling}
                        onMouseLeave={stopScrolling}
                        onTouchStart={() =>
                          canScrollRight && startScrolling("right")
                        }
                        onTouchEnd={stopScrolling}
                        className={cn(
                          "bg-background text-primary absolute right-0 z-50 flex h-6 w-6 items-center justify-center rounded-full border transition-all duration-200",
                          canScrollRight
                            ? "text-muted-foreground hover:text-accent-foreground cursor-pointer"
                            : "hidden!",
                        )}
                        aria-label="Scroll workspaces right"
                        aria-disabled={!canScrollRight}
                      >
                        <ChevronRight className="h-3 w-3" />
                      </motion.button>
                    </div>

                    <motion.div
                      whileHover={{ scale: 1.05 }}
                      whileTap={{ scale: 0.95 }}
                      drag
                      dragConstraints={{ left: 0, right: 0, top: 0, bottom: 0 }}
                      dragElastic={0.1}
                      transition={{
                        type: "spring",
                        stiffness: 400,
                        damping: 25,
                        mass: 0.5,
                      }}
                    >
                      <DropdownMenu
                        open={plusMenuOpen}
                        onOpenChange={setPlusMenuOpen}
                      >
                        <DropdownMenuTrigger asChild>
                          <Button
                            variant="ghost"
                            size="icon"
                            className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
                            onClick={(e: any) => {
                              e.stopPropagation();
                              handleCreateWorkspace({
                                name: "",
                                icon: { type: "dot", value: "" },
                                color:
                                  COLORS[workspaces.length % COLORS.length],
                              });
                            }}
                            onContextMenu={(e: any) => {
                              e.preventDefault();
                              e.stopPropagation();
                              setPlusMenuOpen(true);
                            }}
                          >
                            <Plus className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent
                          side="top"
                          align="end"
                          sideOffset={5}
                          className="border-border bg-card w-56"
                        >
                          <DropdownMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                            <Folder className="mr-2 h-4 w-4" />
                            Live Folder
                            <ChevronRight className="ml-auto h-4 w-4" />
                          </DropdownMenuItem>
                          <div className="border-border my-1 border-t" />
                          <DropdownMenuItem
                            onClick={() => {
                              setWorkspaceDialogOpen(true);
                              setPlusMenuOpen(false);
                            }}
                            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                          >
                            <Copy className="mr-2 h-4 w-4" />
                            Create Space
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            onClick={() => {
                              createFolder();
                              setPlusMenuOpen(false);
                            }}
                            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                          >
                            <Folder className="mr-2 h-4 w-4" />
                            Create Folder
                          </DropdownMenuItem>
                          <div className="border-border my-1 border-t" />
                          <DropdownMenuItem className="text-accent-foreground focus:bg-accent focus:text-accent-foreground">
                            <Columns2 className="mr-2 h-4 w-4" />
                            New Split
                          </DropdownMenuItem>
                          <div className="border-border my-1 border-t" />
                          <DropdownMenuItem
                            onClick={() => {
                              addNewTab();
                              setPlusMenuOpen(false);
                            }}
                            className="text-accent-foreground focus:bg-accent focus:text-accent-foreground"
                          >
                            <Plus className="mr-2 h-4 w-4" />
                            New Tab
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </motion.div>
                  </motion.div>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>

        <div className="flex flex-1 flex-col bg-card border-l p-2">
          {/* <div className="border-border bg-card flex h-11 shrink-0 items-center gap-2 border-b px-4">
          <div className="flex flex-1 items-center gap-2">
            <button
              onClick={() => setCommandOpen(true)}
              className="bg-accent/50 hover:bg-accent text-accent-foreground placeholder:text-muted-foreground focus-visible:ring-ring flex h-9 w-full items-center gap-2 rounded-md border-0 px-3 text-sm transition-colors"
            >
              <Search className="text-muted-foreground h-4 w-4 shrink-0" />
              <span className="text-muted-foreground flex-1 text-left">
                Search or enter address...
              </span>
              <div className="flex items-center gap-1">
                <Link className="text-muted-foreground h-4 w-4" />
                <Grid3x3 className="text-muted-foreground h-4 w-4" />
              </div>
            </button>
          </div>
          <Button
            variant="ghost"
            size="icon"
            className="text-muted-foreground hover:bg-accent hover:text-accent-foreground h-8 w-8"
          >
            <MoreHorizontal className="h-4 w-4" />
          </Button>
        </div> */}

          <div className="flex flex-1 items-center justify-center rounded-md bg-background border">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.5 }}
              className="text-center space-y-6"
            >
              <div>
                <h1 className="text-accent-foreground mb-2 text-4xl font-bold">
                  Zen Browser
                </h1>
                <p className="text-muted-foreground">
                  Experience the web with vertical tabs and workspaces
                </p>
              </div>
              <ThemeSwitcher />
              <DockDemo />
            </motion.div>
          </div>
        </div>
      </div>

      <WorkspaceDialog
        open={workspaceDialogOpen}
        onOpenChange={(open) => {
          setWorkspaceDialogOpen(open);
          if (!open) {
            setWorkspaceEditMode(undefined);
          }
        }}
        onCreateWorkspace={handleCreateWorkspace}
        editMode={workspaceEditMode}
        onUpdateWorkspace={handleUpdateWorkspace}
      />

      <Dialog open={commandOpen} onOpenChange={setCommandOpen}>
        <DialogContent className="overflow-hidden p-0 shadow-lg">
          <VisuallyHidden>
            <DialogTitle>Search and Command Palette</DialogTitle>
          </VisuallyHidden>
          <Command className="**:[[cmdk-group-heading]]:text-muted-foreground **:[[cmdk-group-heading]]:px-2 **:[[cmdk-group-heading]]:font-medium [&_[cmdk-group]:not([hidden])_~[cmdk-group]]:pt-0 **:[[cmdk-group]]:px-2 [&_[cmdk-input-wrapper]_svg]:h-5 [&_[cmdk-input-wrapper]_svg]:w-5 **:[[cmdk-input]]:h-12 **:[[cmdk-item]]:px-2 **:[[cmdk-item]]:py-3 [&_[cmdk-item]_svg]:h-5 [&_[cmdk-item]_svg]:w-5">
            <CommandInput placeholder="Search or enter address..." />
            <CommandList>
              <CommandEmpty>No results found.</CommandEmpty>
              <CommandGroup heading="Suggestions">
                <CommandItem
                  onSelect={() => {
                    addNewTab();
                    setCommandOpen(false);
                  }}
                >
                  <Plus className="mr-2 h-4 w-4" />
                  <span>New Tab</span>
                </CommandItem>
                <CommandItem
                  onSelect={() => {
                    setWorkspaceDialogOpen(true);
                    setCommandOpen(false);
                  }}
                >
                  <Copy className="mr-2 h-4 w-4" />
                  <span>Create Space</span>
                </CommandItem>
                <CommandItem
                  onSelect={() => {
                    clearAllTabs();
                    setCommandOpen(false);
                  }}
                >
                  <X className="mr-2 h-4 w-4" />
                  <span>Clear All Tabs</span>
                </CommandItem>
              </CommandGroup>
              <CommandGroup heading="Recent Tabs">
                {activeWorkspaceTabs.slice(0, 5).map((tab) => (
                  <CommandItem
                    key={tab.id}
                    onSelect={() => {
                      setActiveTab(tab.id);
                      setCommandOpen(false);
                    }}
                  >
                    <div className="bg-destructive mr-2 h-4 w-4 shrink-0 rounded-sm" />
                    <span className="max-w-[75%] truncate">{tab.title}</span>
                  </CommandItem>
                ))}
              </CommandGroup>
              <CommandGroup heading="Workspaces">
                {workspaces.map((workspace) => (
                  <CommandItem
                    key={workspace.id}
                    onSelect={() => {
                      setActiveWorkspace(workspace.id);
                      setCommandOpen(false);
                    }}
                  >
                    <div className="mr-2 flex h-4 w-4 items-center justify-center">
                      {renderWorkspaceIcon(
                        workspace,
                        activeWorkspace === workspace.id,
                      )}
                    </div>
                    <span className="max-w-[75%] truncate">
                      {workspace.name}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </DialogContent>
      </Dialog>

      <DragOverlay>
        {activeId ? (
          <div className="bg-accent text-accent-foreground flex h-9 items-center gap-2 rounded-md px-2 shadow-lg">
            <div className="bg-destructive h-4 w-4 shrink-0 rounded-sm" />
            <span className="text-xs">
              {looseTabs.find((t) => t.id === activeId)?.title || "Dragging..."}
            </span>
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
