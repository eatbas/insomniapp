import { Github, Heart } from 'lucide-react'

export function Footer() {
  return (
    <footer className="border-t border-white/5 py-12 px-6">
      <div className="max-w-6xl mx-auto">
        <div className="flex flex-col md:flex-row items-center justify-between gap-6">
          <div className="flex items-center gap-2">
            <img src="/logo.png" alt="insomniAPP" className="w-6 h-6" />
            <span className="font-semibold">
              insomni<span className="text-primary">APP</span>
            </span>
          </div>

          <div className="flex items-center gap-6 text-sm text-slate-500">
            <a
              href="https://github.com/eatbas/insomniapp"
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center gap-2 hover:text-slate-300 transition-colors"
            >
              <Github className="w-4 h-4" />
              GitHub
            </a>
            <a
              href="https://github.com/eatbas/insomniapp/releases"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors"
            >
              Releases
            </a>
            <a
              href="https://github.com/eatbas/insomniapp/issues"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-slate-300 transition-colors"
            >
              Issues
            </a>
          </div>

          <div className="flex items-center gap-1 text-sm text-slate-500">
            Made with <Heart className="w-3.5 h-3.5 text-red-500" /> using Tauri + React + Rust
          </div>
        </div>

        <div className="mt-8 text-center text-xs text-slate-600">
          MIT License &middot; &copy; {new Date().getFullYear()} insomniAPP
        </div>
      </div>
    </footer>
  )
}
