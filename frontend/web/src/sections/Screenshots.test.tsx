import { describe, it, expect } from 'vitest'
import { render, screen, within } from '@testing-library/react'
import { Screenshots } from './Screenshots'

describe('Screenshots', () => {
  it('renders the section heading and both theme screenshots', () => {
    const { container } = render(<Screenshots />)

    expect(container.querySelector('section#screenshots')).not.toBeNull()
    expect(screen.getByText('Dark Mode')).toBeInTheDocument()
    expect(screen.getByText('Light Mode')).toBeInTheDocument()
    expect(
      screen.getByRole('img', { name: 'insomniAPP dark mode' }),
    ).toBeInTheDocument()
    expect(
      screen.getByRole('img', { name: 'insomniAPP light mode' }),
    ).toBeInTheDocument()
  })

  it('renders the dark card before the light card, each with its own caption', () => {
    render(<Screenshots />)

    const images = screen.getAllByRole('img')
    expect(images).toHaveLength(2)

    const expectedCards = [
      { src: '/ss1.png', alt: 'insomniAPP dark mode', caption: 'Dark Mode' },
      { src: '/ss2.png', alt: 'insomniAPP light mode', caption: 'Light Mode' },
    ]

    expectedCards.forEach(({ src, alt, caption }, index) => {
      const image = images[index]
      expect(image).toHaveAttribute('src', src)
      expect(image).toHaveAttribute('alt', alt)

      const card = image.closest<HTMLElement>('.group')
      expect(card).not.toBeNull()
      expect(within(card as HTMLElement).getByText(caption)).toBeInTheDocument()
    })
  })
})
