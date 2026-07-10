import { describe, it, expect, vi, beforeEach } from 'vitest'

// `createRoot` is stubbed so the entrypoint can be executed without mounting the
// real application (which would trigger network effects) while still asserting
// that the bootstrap wires the root element to a render call.
const { createRootSpy, renderSpy } = vi.hoisted(() => {
  const renderSpy = vi.fn()
  const createRootSpy = vi.fn(() => ({ render: renderSpy }))
  return { createRootSpy, renderSpy }
})

vi.mock('react-dom/client', () => ({ createRoot: createRootSpy }))

describe('main entrypoint', () => {
  beforeEach(() => {
    vi.resetModules()
    createRootSpy.mockClear()
    renderSpy.mockClear()
    document.body.innerHTML = '<div id="root"></div>'
  })

  it('creates a root on #root and renders the app once', async () => {
    await import('./main')

    expect(createRootSpy).toHaveBeenCalledWith(document.getElementById('root'))
    expect(renderSpy).toHaveBeenCalledTimes(1)
  })
})
