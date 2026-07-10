import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import App from './App'

describe('App', () => {
  it('composes the landing page from every major section', () => {
    // Both Hero and Download fetch on mount; a never-resolving fetch keeps the
    // render deterministic and free of act warnings.
    vi.stubGlobal('fetch', vi.fn().mockReturnValue(new Promise(() => {})))

    render(<App />)

    expect(screen.getByRole('navigation')).toBeInTheDocument()
    expect(
      screen.getByRole('heading', { level: 1, name: /Keep Your Computer/i }),
    ).toBeInTheDocument()
    expect(document.querySelector('section#features')).not.toBeNull()
    expect(document.querySelector('section#screenshots')).not.toBeNull()
    expect(document.querySelector('section#download')).not.toBeNull()
    expect(document.querySelector('footer')).not.toBeNull()
  })
})
