interface SectionHeadingProps {
  badge: string
  title: string
  description: string
}

export function SectionHeading({ badge, title, description }: SectionHeadingProps) {
  return (
    <div className="text-center mb-16">
      <span className="inline-block px-4 py-1.5 rounded-full text-xs font-medium tracking-wider uppercase bg-primary/10 text-primary border border-primary/20 mb-4">
        {badge}
      </span>
      <h2 className="text-3xl md:text-4xl font-bold mb-4">{title}</h2>
      <p className="text-slate-400 max-w-2xl mx-auto text-lg">{description}</p>
    </div>
  )
}
