import { SectionHeading } from '../components/SectionHeading'

export function Screenshots() {
  return (
    <section id="screenshots" className="py-24 px-6">
      <div className="max-w-6xl mx-auto">
        <SectionHeading
          badge="Screenshots"
          title="Compact & Clean Interface"
          description="An ultra-compact 240x78px window that stays out of your way. Dark and light themes included."
        />

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-3xl mx-auto">
          {/* Dark mode screenshot */}
          <div className="group">
            <div className="glass rounded-2xl p-6 transition-all duration-300 group-hover:shadow-xl group-hover:shadow-primary/5">
              <div className="rounded-lg overflow-hidden shadow-2xl mb-4">
                <img
                  src="/ss1.png"
                  alt="insomniAPP dark mode"
                  className="w-full h-auto"
                />
              </div>
              <p className="text-center text-sm text-slate-400">Dark Mode</p>
            </div>
          </div>

          {/* Light mode screenshot */}
          <div className="group">
            <div className="glass rounded-2xl p-6 transition-all duration-300 group-hover:shadow-xl group-hover:shadow-primary/5">
              <div className="rounded-lg overflow-hidden shadow-2xl mb-4">
                <img
                  src="/ss2.png"
                  alt="insomniAPP light mode"
                  className="w-full h-auto"
                />
              </div>
              <p className="text-center text-sm text-slate-400">Light Mode</p>
            </div>
          </div>
        </div>
      </div>
    </section>
  )
}
