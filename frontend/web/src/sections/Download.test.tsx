import { describe, it, expect, vi } from 'vitest'
import { render, screen, act, waitFor } from '@testing-library/react'
import { Download } from './Download'

const RELEASES = 'https://github.com/eatbas/insomniapp/releases/latest'

function windowsLink() {
  return screen.getByRole('link', { name: /Download for Windows/i })
}

function macosLink() {
  return screen.getByRole('link', { name: /Download for macOS/i })
}

function mockJson(payload: unknown, ok = true) {
  vi.stubGlobal(
    'fetch',
    vi.fn().mockResolvedValue({ ok, json: async () => payload }),
  )
}

describe('Download', () => {
  it('defaults both platform links to the releases page before any data loads', () => {
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))
    render(<Download />)

    expect(windowsLink()).toHaveAttribute('href', RELEASES)
    expect(macosLink()).toHaveAttribute('href', RELEASES)
  })

  it('selects the Windows setup asset and the universal macOS dmg', async () => {
    mockJson({
      assets: [
        { name: 'insomniAPP_1.0.0_x64-setup.exe', browser_download_url: 'https://win' },
        { name: 'insomniAPP_1.0.0_universal.dmg', browser_download_url: 'https://mac-universal' },
      ],
    })
    render(<Download />)

    await waitFor(() => expect(windowsLink()).toHaveAttribute('href', 'https://win'))
    expect(macosLink()).toHaveAttribute('href', 'https://mac-universal')
  })

  it('falls back to a generic macOS dmg when no universal build exists', async () => {
    mockJson({
      assets: [
        { name: 'insomniAPP_1.0.0_x64-setup.exe', browser_download_url: 'https://win' },
        { name: 'insomniAPP_1.0.0_aarch64.dmg', browser_download_url: 'https://mac-generic' },
      ],
    })
    render(<Download />)

    await waitFor(() => expect(macosLink()).toHaveAttribute('href', 'https://mac-generic'))
    expect(windowsLink()).toHaveAttribute('href', 'https://win')
  })

  it('falls back to the releases page when the payload has no assets', async () => {
    mockJson({})
    render(<Download />)

    await act(async () => {})

    expect(windowsLink()).toHaveAttribute('href', RELEASES)
    expect(macosLink()).toHaveAttribute('href', RELEASES)
  })

  it('keeps default links when the API returns a non-OK response', async () => {
    const jsonSpy = vi.fn()
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, json: jsonSpy }))
    render(<Download />)

    await act(async () => {})

    expect(jsonSpy).not.toHaveBeenCalled()
    expect(windowsLink()).toHaveAttribute('href', RELEASES)
  })

  it('silently ignores an aborted request', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockRejectedValue(new DOMException('aborted', 'AbortError')),
    )
    render(<Download />)

    await act(async () => {})

    expect(windowsLink()).toHaveAttribute('href', RELEASES)
  })

  it('swallows a non-abort DOMException without updating links', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockRejectedValue(new DOMException('nope', 'NotSupportedError')),
    )
    render(<Download />)

    await act(async () => {})

    expect(macosLink()).toHaveAttribute('href', RELEASES)
  })

  it('swallows a generic rejected fetch without updating links', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('boom')))
    render(<Download />)

    await act(async () => {})

    expect(windowsLink()).toHaveAttribute('href', RELEASES)
  })

  it('aborts the in-flight request when unmounted', () => {
    const abortSpy = vi.spyOn(AbortController.prototype, 'abort')
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    const { unmount } = render(<Download />)
    unmount()

    expect(abortSpy).toHaveBeenCalled()
  })
})
