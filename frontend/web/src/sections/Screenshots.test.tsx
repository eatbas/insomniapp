import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
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
})
