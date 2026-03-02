import { useState, useEffect } from 'react'
import { Menu, X } from 'lucide-react'

const links = [
  { href: '#features', label: 'Features' },
  { href: '#screenshots', label: 'Screenshots' },
  { href: '#download', label: 'Download' },
]

export function Navbar() {
  const [scrolled, setScrolled] = useState(false)
  const [mobileOpen, setMobileOpen] = useState(false)

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 20)
    window.addEventListener('scroll', onScroll)
    return () => window.removeEventListener('scroll', onScroll)
  }, [])

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled ? 'glass shadow-lg' : 'bg-transparent'
      }`}
    >
      <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
        <a href="#" className="flex items-center gap-2 group">
          <img src="/logo.png" alt="insomniAPP" className="w-8 h-8" />
          <span className="font-bold text-lg tracking-tight">
            insomni<span className="text-primary">APP</span>
          </span>
        </a>

        <div className="hidden md:flex items-center gap-8">
          {links.map((link) => (
            <a
              key={link.href}
              href={link.href}
              className="text-sm text-slate-400 hover:text-white transition-colors"
            >
              {link.label}
            </a>
          ))}
          <a
            href="https://github.com/eatbas/insomniapp"
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm px-4 py-2 rounded-lg bg-primary/10 text-primary hover:bg-primary/20 transition-colors"
          >
            GitHub
          </a>
        </div>

        <button
          className="md:hidden text-slate-400 hover:text-white"
          onClick={() => setMobileOpen(!mobileOpen)}
        >
          {mobileOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
        </button>
      </div>

      {mobileOpen && (
        <div className="md:hidden glass border-t border-white/5">
          <div className="px-6 py-4 flex flex-col gap-4">
            {links.map((link) => (
              <a
                key={link.href}
                href={link.href}
                className="text-sm text-slate-400 hover:text-white transition-colors"
                onClick={() => setMobileOpen(false)}
              >
                {link.label}
              </a>
            ))}
            <a
              href="https://github.com/eatbas/insomniapp"
              target="_blank"
              rel="noopener noreferrer"
              className="text-sm text-primary hover:text-primary-light transition-colors"
              onClick={() => setMobileOpen(false)}
            >
              GitHub
            </a>
          </div>
        </div>
      )}
    </nav>
  )
}
