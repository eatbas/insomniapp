import type { ReactNode } from 'react'

interface FeatureCardProps {
  icon: ReactNode
  title: string
  description: string
  accent?: string
}

export function FeatureCard({ icon, title, description, accent = 'primary' }: FeatureCardProps) {
  const accentMap: Record<string, string> = {
    primary: 'group-hover:border-primary/30 group-hover:shadow-primary/5',
    green: 'group-hover:border-accent-green/30 group-hover:shadow-accent-green/5',
    blue: 'group-hover:border-accent-blue/30 group-hover:shadow-accent-blue/5',
    yellow: 'group-hover:border-accent-yellow/30 group-hover:shadow-accent-yellow/5',
  }

  const iconBgMap: Record<string, string> = {
    primary: 'bg-primary/10 text-primary',
    green: 'bg-accent-green/10 text-accent-green',
    blue: 'bg-accent-blue/10 text-accent-blue',
    yellow: 'bg-accent-yellow/10 text-accent-yellow',
  }

  return (
    <div
      className={`group glass glass-hover rounded-2xl p-6 transition-all duration-300 hover:shadow-xl ${accentMap[accent]}`}
    >
      <div className={`w-12 h-12 rounded-xl flex items-center justify-center mb-4 ${iconBgMap[accent]}`}>
        {icon}
      </div>
      <h3 className="text-lg font-semibold mb-2">{title}</h3>
      <p className="text-slate-400 text-sm leading-relaxed">{description}</p>
    </div>
  )
}
