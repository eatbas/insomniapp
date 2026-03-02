import { Download as DownloadIcon, Monitor, Apple } from 'lucide-react'

const GITHUB_RELEASES = 'https://github.com/eatbas/insomniapp/releases/latest'

export function Download() {
  return (
    <section id="download" className="py-24 px-6">
      <div className="max-w-4xl mx-auto text-center">
        {/* Background glow */}
        <div className="relative">
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-96 h-96 bg-primary/10 rounded-full blur-[120px] pointer-events-none" />

          <div className="relative z-10">
            <span className="inline-block px-4 py-1.5 rounded-full text-xs font-medium tracking-wider uppercase bg-primary/10 text-primary border border-primary/20 mb-4">
              Download
            </span>
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              Get insomniAPP
            </h2>
            <p className="text-slate-400 text-lg mb-10 max-w-xl mx-auto">
              Download the latest release for your platform. Lightweight, native, and
              ready to keep your computer awake in seconds.
            </p>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-8">
              <a
                href={`${GITHUB_RELEASES}/download/insomniapp_0.1.0_x64-setup.exe`}
                className="group flex items-center gap-3 px-8 py-4 rounded-xl bg-gradient-to-r from-primary to-primary-dark text-white font-semibold hover:shadow-lg hover:shadow-primary/25 transition-all duration-300 hover:-translate-y-0.5 w-full sm:w-auto justify-center"
              >
                <Monitor className="w-5 h-5" />
                Download for Windows
                <span className="text-xs opacity-60">.exe</span>
              </a>
              <a
                href={`${GITHUB_RELEASES}/download/insomniapp_0.1.0_aarch64.dmg`}
                className="group flex items-center gap-3 px-8 py-4 rounded-xl glass glass-hover font-semibold transition-all duration-300 hover:-translate-y-0.5 w-full sm:w-auto justify-center"
              >
                <Apple className="w-5 h-5" />
                Download for macOS
                <span className="text-xs opacity-60">.dmg</span>
              </a>
            </div>

            <a
              href={GITHUB_RELEASES}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 text-sm text-slate-400 hover:text-primary transition-colors"
            >
              <DownloadIcon className="w-4 h-4" />
              View all releases on GitHub
            </a>
          </div>
        </div>
      </div>
    </section>
  )
}
