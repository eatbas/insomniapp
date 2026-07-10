import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { Footer } from './Footer'

describe('Footer', () => {
  it('renders the brand, navigation links and current-year copyright', () => {
    const { container } = render(<Footer />)

    expect(container.querySelector('footer')).not.toBeNull()

    const github = screen.getByRole('link', { name: /GitHub/i })
    expect(github).toHaveAttribute('href', 'https://github.com/eatbas/insomniapp')
    expect(github).toHaveAttribute('target', '_blank')
    expect(github).toHaveAttribute('rel', 'noopener noreferrer')

    expect(
      screen.getByRole('link', { name: 'Releases' }),
    ).toHaveAttribute('href', 'https://github.com/eatbas/insomniapp/releases')
    expect(
      screen.getByRole('link', { name: 'Issues' }),
    ).toHaveAttribute('href', 'https://github.com/eatbas/insomniapp/issues')

    const year = new Date().getFullYear().toString()
    expect(
      screen.getByText((content) => content.includes(year)),
    ).toBeInTheDocument()
  })
})
