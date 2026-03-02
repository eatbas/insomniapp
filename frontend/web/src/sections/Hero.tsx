import { Download, Github, ChevronDown } from 'lucide-react'

export function Hero() {
  return (
    <section className="relative min-h-screen flex items-center justify-center overflow-hidden">
      {/* Background effects */}
      <div className="absolute inset-0">
        <div className="absolute top-1/4 left-1/4 w-96 h-96 bg-primary/10 rounded-full blur-[120px]" />
        <div className="absolute bottom-1/4 right-1/4 w-80 h-80 bg-primary-dark/10 rounded-full blur-[100px]" />
        <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[600px] bg-primary/5 rounded-full blur-[150px]" />
      </div>

      {/* Grid overlay */}
      <div
        className="absolute inset-0 opacity-[0.03]"
        style={{
          backgroundImage:
            'linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)',
          backgroundSize: '60px 60px',
        }}
      />

      <div className="relative z-10 max-w-4xl mx-auto px-6 text-center pt-24">
        {/* Floating logo */}
        <div className="animate-float mb-8 inline-block">
          <div className="animate-pulse-glow rounded-3xl">
            <img src="/logo.png" alt="insomniAPP" className="w-28 h-28 drop-shadow-2xl" />
          </div>
        </div>

        {/* Badge */}
        <div className="mb-6">
          <span className="inline-flex items-center gap-2 px-4 py-2 rounded-full text-base bg-primary/10 text-primary border border-primary/20">
            <span className="w-2 h-2 rounded-full bg-accent-green animate-pulse" />
            v0.1.0 — Built with Tauri 2 + React + Rust
          </span>
        </div>

        {/* Heading */}
        <h1 className="text-5xl md:text-7xl font-bold tracking-tight mb-6">
          Keep Your Computer{' '}
          <span className="bg-gradient-to-r from-primary-light to-primary bg-clip-text text-transparent animate-gradient">
            Awake
          </span>
        </h1>

        {/* Subheading */}
        <p className="text-xl md:text-2xl text-slate-400 max-w-2xl mx-auto mb-10 leading-relaxed">
          A lightweight desktop app that prevents sleep, screen lock, and away status
          — by silently pressing F15 when you're idle.{' '}
          <span className="text-slate-300">Smart enough to pause during meetings.</span>
        </p>

        {/* CTA buttons */}
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 mb-16">
          <a
            href="#download"
            className="group flex items-center gap-2 px-8 py-4 rounded-xl bg-gradient-to-r from-primary to-primary-dark text-white font-semibold hover:shadow-lg hover:shadow-primary/25 transition-all duration-300 hover:-translate-y-0.5"
          >
            <Download className="w-5 h-5" />
            Download Now
          </a>
          <a
            href="https://github.com/eatbas/insomniapp"
            target="_blank"
            rel="noopener noreferrer"
            className="group flex items-center gap-2 px-8 py-4 rounded-xl glass glass-hover font-semibold transition-all duration-300 hover:-translate-y-0.5"
          >
            <Github className="w-5 h-5" />
            View on GitHub
          </a>
        </div>

        {/* Status indicators preview */}
        <div className="glass rounded-2xl p-6 max-w-md mx-auto">
          <p className="text-xs text-slate-500 uppercase tracking-wider mb-3">Status Indicators</p>
          <div className="flex justify-center gap-6">
            {[
              { color: 'bg-accent-gray', label: 'Disabled' },
              { color: 'bg-accent-yellow', label: 'In Meeting' },
              { color: 'bg-accent-blue', label: 'Monitoring' },
              { color: 'bg-accent-green animate-pulse', label: 'Active' },
            ].map(({ color, label }) => (
              <div key={label} className="flex items-center gap-2">
                <span className={`w-2.5 h-2.5 rounded-full ${color}`} />
                <span className="text-xs text-slate-400">{label}</span>
              </div>
            ))}
          </div>
        </div>

        {/* Scroll indicator */}
        <div className="mt-16 animate-bounce">
          <ChevronDown className="w-6 h-6 text-slate-500 mx-auto" />
        </div>
      </div>
    </section>
  )
}
