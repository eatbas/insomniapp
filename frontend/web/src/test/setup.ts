import '@testing-library/jest-dom/vitest'
import { afterEach, vi } from 'vitest'
import { cleanup } from '@testing-library/react'

// Unmount React trees, reset the jsdom document, and clear any globals stubbed
// with `vi.stubGlobal` (for example `fetch`) between tests so that component
// state, event listeners, rendered markup, and network mocks never leak across
// cases. Testing Library does not auto-cleanup unless globals are enabled and
// this file is loaded as a setup file.
afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})
