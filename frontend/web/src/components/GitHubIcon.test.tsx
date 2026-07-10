import { describe, it, expect } from 'vitest'
import { render } from '@testing-library/react'
import { GitHubIcon } from './GitHubIcon'

describe('GitHubIcon', () => {
  it('renders an accessible-hidden svg and forwards props', () => {
    const { container } = render(<GitHubIcon className="w-5 h-5" data-testid="gh" />)

    const svg = container.querySelector('svg')
    expect(svg).not.toBeNull()
    expect(svg).toHaveClass('w-5', 'h-5')
    expect(svg).toHaveAttribute('data-testid', 'gh')
    expect(svg).toHaveAttribute('aria-hidden', 'true')
    expect(svg?.querySelector('path')).not.toBeNull()
  })
})
