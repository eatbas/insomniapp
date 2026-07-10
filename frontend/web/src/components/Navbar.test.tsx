import { describe, it, expect, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { Navbar } from './Navbar'

function setScrollY(value: number) {
  Object.defineProperty(window, 'scrollY', { configurable: true, value })
}

describe('Navbar', () => {
  beforeEach(() => {
    setScrollY(0)
  })

  it('starts transparent and becomes glass once scrolled past the threshold', () => {
    render(<Navbar />)
    const nav = screen.getByRole('navigation')
    expect(nav).toHaveClass('bg-transparent')

    setScrollY(30)
    fireEvent.scroll(window)

    expect(nav).toHaveClass('glass', 'shadow-lg')
    expect(nav).not.toHaveClass('bg-transparent')
  })

  it('exposes the GitHub call-to-action as an external link', () => {
    render(<Navbar />)
    const github = screen.getByRole('link', { name: 'GitHub' })
    expect(github).toHaveAttribute('href', 'https://github.com/eatbas/insomniapp')
    expect(github).toHaveAttribute('target', '_blank')
    expect(github).toHaveAttribute('rel', 'noopener noreferrer')
  })

  it('opens and closes the mobile menu with the toggle button', async () => {
    const user = userEvent.setup()
    render(<Navbar />)

    // Desktop links are always present; the mobile menu duplicates them.
    expect(screen.getAllByRole('link', { name: 'Features' })).toHaveLength(1)

    await user.click(screen.getByRole('button'))
    expect(screen.getAllByRole('link', { name: 'Features' })).toHaveLength(2)

    await user.click(screen.getByRole('button'))
    expect(screen.getAllByRole('link', { name: 'Features' })).toHaveLength(1)
  })

  it('closes the mobile menu when a mobile navigation link is clicked', async () => {
    const user = userEvent.setup()
    render(<Navbar />)

    await user.click(screen.getByRole('button'))
    const mobileFeatures = screen.getAllByRole('link', { name: 'Features' })[1]

    await user.click(mobileFeatures)
    expect(screen.getAllByRole('link', { name: 'Features' })).toHaveLength(1)
  })

  it('closes the mobile menu when the mobile GitHub link is clicked', async () => {
    const user = userEvent.setup()
    render(<Navbar />)

    await user.click(screen.getByRole('button'))
    // While open there are two GitHub links (desktop + mobile); the second is
    // the mobile one whose click handler also collapses the menu.
    const mobileGithub = screen.getAllByRole('link', { name: 'GitHub' })[1]

    await user.click(mobileGithub)
    expect(screen.getAllByRole('link', { name: 'GitHub' })).toHaveLength(1)
  })
})
