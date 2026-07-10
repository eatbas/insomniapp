import { describe, it, expect, vi } from 'vitest'
import { render, screen, act } from '@testing-library/react'
import { Hero } from './Hero'

describe('Hero', () => {
  it('renders the static hero content without a version before the fetch resolves', () => {
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))
    render(<Hero />)

    expect(
      screen.getByRole('heading', { level: 1, name: /Keep Your Computer/i }),
    ).toBeInTheDocument()
    expect(
      screen.getByText('Built with Tauri 2 + React + Rust'),
    ).toBeInTheDocument()
  })

  it('shows the latest release tag when the API responds successfully', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue({
        ok: true,
        json: async () => ({ tag_name: 'v9.9.9' }),
      }),
    )
    render(<Hero />)

    expect(await screen.findByText(/v9\.9\.9 —/)).toBeInTheDocument()
  })

  it('falls back to no version when the API returns a non-OK response', async () => {
    const jsonSpy = vi.fn()
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue({ ok: false, json: jsonSpy }))
    render(<Hero />)

    await act(async () => {})

    expect(jsonSpy).not.toHaveBeenCalled()
    expect(
      screen.getByText('Built with Tauri 2 + React + Rust'),
    ).toBeInTheDocument()
    expect(screen.queryByText(/v\d+\.\d+\.\d+ —/)).not.toBeInTheDocument()
  })

  it('swallows a rejected fetch and keeps the version empty', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')))
    render(<Hero />)

    await act(async () => {})

    expect(
      screen.getByText('Built with Tauri 2 + React + Rust'),
    ).toBeInTheDocument()
  })

  it('aborts the in-flight request when unmounted', () => {
    const abortSpy = vi.spyOn(AbortController.prototype, 'abort')
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    const { unmount } = render(<Hero />)
    unmount()

    expect(abortSpy).toHaveBeenCalled()
  })
})
