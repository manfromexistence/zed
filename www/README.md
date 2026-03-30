# Magic UI Demo - Next.js 16 + Tailwind CSS v4

git clone https://github.com/manfromexistence/friday && cd friday && rm -rf .git && cd ...
git clone https://github.com/pqoqubbw/icons && cd friday && rm -rf .git && cd ...
git clone https://github.com/Avijit07x/animateicons && cd friday && rm -rf .git && cd ...

https://github.com/pqoqubbw/icons
https://github.com/Avijit07x/animateicons

A modern web application showcasing Magic UI components with the latest tech stack.

## Tech Stack

- **Next.js** v16.1.6 - React framework with App Router
- **React** v19.2.3 - UI library
- **Tailwind CSS** v4.2.1 - Utility-first CSS framework
- **shadcn-ui** - Accessible component library
- **Magic UI** - Animated components and effects
- **Framer Motion** v12.36.0 - Animation library
- **cmdk** v1.1.1 - Command palette component

## Getting Started

Install dependencies:

```bash
bun install
```

Run the development server:

```bash
bun run dev
```

Open [http://localhost:3000](http://localhost:3000) in your browser.

## Features

- ✨ Meteors background effect
- 🎆 Border Beam animation
- 💫 Shimmer Button component
- ⌘K Command palette (press Cmd/Ctrl + K)
- 🎨 Dark mode optimized
- 🚀 Built with latest packages (March 2026)

## Adding More Magic UI Components

Add components using the shadcn CLI:

```bash
bunx shadcn@latest add "https://magicui.design/r/[component-name]"
```

Available components:
- `magic-card` - Animated card with gradient effects
- `animated-beam` - Connecting beam animations
- `globe` - Interactive 3D globe
- `marquee` - Scrolling text/logo marquee
- `confetti` - Celebration confetti effect
- `particles` - Particle system background
- `shine-border` - Animated border shine
- `orbiting-circles` - Orbiting elements animation
- `dock` - macOS-style dock
- `bento-grid` - Modern grid layout
- `animated-list` - List with enter/exit animations

## Project Structure

```
www/
├── src/
│   ├── app/
│   │   ├── globals.css
│   │   ├── layout.tsx
│   │   └── page.tsx
│   ├── components/
│   │   ├── ui/              # shadcn-ui components
│   │   └── magicui/         # Magic UI components
│   ├── hooks/
│   │   └── useCommandK.ts
│   └── lib/
│       └── utils.ts
├── components.json
└── package.json
```

## Learn More

- [Next.js Documentation](https://nextjs.org/docs)
- [Tailwind CSS v4](https://tailwindcss.com)
- [shadcn-ui](https://ui.shadcn.com)
- [Magic UI](https://magicui.design)
