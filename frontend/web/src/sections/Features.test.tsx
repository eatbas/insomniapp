import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Features } from './Features'

describe('Features', () => {
  it('renders the heading and one card per feature', () => {
    const { container } = render(<Features />)

    expect(container.querySelector('section#features')).not.toBeNull()
    expect(
      screen.getByRole('heading', {
        level: 2,
        name: 'Everything You Need to Stay Active',
      }),
    ).toBeInTheDocument()

    for (const title of [
      'Keep-Awake Engine',
      'System Tray Integration',
      'Real-Time Status',
      'Configurable Settings',
      'Dark & Light Themes',
    ]) {
      expect(
        screen.getByRole('heading', { level: 3, name: title }),
      ).toBeInTheDocument()
    }
  })
})
