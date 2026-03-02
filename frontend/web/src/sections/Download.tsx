import { useEffect, useState } from 'react'
import { Download as DownloadIcon, Monitor, Apple } from 'lucide-react'

const GITHUB_OWNER = 'eatbas'
const GITHUB_REPO = 'insomniapp'
const GITHUB_RELEASES = 'https://github.com/eatbas/insomniapp/releases/latest'
const GITHUB_API_LATEST = `https://api.github.com/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/latest`

type DownloadLinks = {
  windows: string
  macos: string
}

type ReleaseAsset = {
  name: string
  browser_download_url: string
}

type LatestReleaseResponse = {
  assets?: ReleaseAsset[]
}

function findAssetUrl(assets: ReleaseAsset[], matcher: RegExp): string | null {
  const asset = assets.find(({ name }) => matcher.test(name))
  return asset?.browser_download_url ?? null
}

export function Download() {
  const [links, setLinks] = useState<DownloadLinks>({
    windows: GITHUB_RELEASES,
    macos: GITHUB_RELEASES,
  })

  useEffect(() => {
    const controller = new AbortController()

    const loadLatestReleaseAssets = async () => {
      try {
        const response = await fetch(GITHUB_API_LATEST, {
          headers: { Accept: 'application/vnd.github+json' },
          signal: controller.signal,
        })

        if (!response.ok) return

        const payload = (await response.json()) as LatestReleaseResponse
        const assets = payload.assets ?? []

        const windows = findAssetUrl(assets, /_x64-setup\.exe$/i) ?? GITHUB_RELEASES
        const macos =
          findAssetUrl(assets, /_universal\.dmg$/i) ??
          findAssetUrl(assets, /\.dmg$/i) ??
          GITHUB_RELEASES

        setLinks({ windows, macos })
      } catch (error) {
        if (error instanceof DOMException && error.name === 'AbortError') {
          return
        }
      }
    }

    void loadLatestReleaseAssets()

    return () => {
      controller.abort()
    }
  }, [])

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
                href={links.windows}
                className="group flex items-center gap-3 px-8 py-4 rounded-xl bg-gradient-to-r from-primary to-primary-dark text-white font-semibold hover:shadow-lg hover:shadow-primary/25 transition-all duration-300 hover:-translate-y-0.5 w-full sm:w-auto justify-center"
              >
                <Monitor className="w-5 h-5" />
                Download for Windows
                <span className="text-xs opacity-60">.exe</span>
              </a>
              <a
                href={links.macos}
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
