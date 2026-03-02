import {
  Shield,
  Video,
  MonitorSmartphone,
  Timer,
  Settings,
  Palette,
} from 'lucide-react'
import { SectionHeading } from '../components/SectionHeading'
import { FeatureCard } from '../components/FeatureCard'

const features = [
  {
    icon: <Shield className="w-6 h-6" />,
    title: 'Keep-Awake Engine',
    description:
      'Monitors system idle time every 3 seconds using native OS APIs. Simulates the F15 key — a key virtually never mapped to anything — so it won\'t interfere with your work.',
    accent: 'green',
  },
  {
    icon: <Video className="w-6 h-6" />,
    title: 'Meeting Detection',
    description:
      'Automatically detects active microphone and camera usage. Pauses all activity simulation during meetings — no interference with video calls or screen shares.',
    accent: 'yellow',
  },
  {
    icon: <MonitorSmartphone className="w-6 h-6" />,
    title: 'System Tray Integration',
    description:
      'Lives unobtrusively in the system tray. Left-click to show the compact status window, right-click for a context menu with quick actions.',
    accent: 'blue',
  },
  {
    icon: <Timer className="w-6 h-6" />,
    title: 'Real-Time Status',
    description:
      'Four color-coded states (Disabled, In Meeting, Monitoring, Active) with a live idle timer display and visual progress bar.',
    accent: 'primary',
  },
  {
    icon: <Settings className="w-6 h-6" />,
    title: 'Configurable Settings',
    description:
      'Adjust idle threshold (10–600s) and simulation interval (5–300s). Changes apply instantly with a 500ms debounce — no restart needed.',
    accent: 'green',
  },
  {
    icon: <Palette className="w-6 h-6" />,
    title: 'Dark & Light Themes',
    description:
      'Toggle between dark and light mode with a single click. The ultra-compact window (240x78px) stays out of your way at the bottom-left corner.',
    accent: 'blue',
  },
]

export function Features() {
  return (
    <section id="features" className="py-24 px-6">
      <div className="max-w-6xl mx-auto">
        <SectionHeading
          badge="Features"
          title="Everything You Need to Stay Active"
          description="Designed to be invisible. insomniAPP works silently in the background with smart detection and zero interference."
        />
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {features.map((feature) => (
            <FeatureCard key={feature.title} {...feature} />
          ))}
        </div>
      </div>
    </section>
  )
}
