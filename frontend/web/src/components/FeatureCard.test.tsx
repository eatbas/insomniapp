import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { FeatureCard } from './FeatureCard'

describe('FeatureCard', () => {
  it('renders icon, title and description with the default primary accent', () => {
    render(
      <FeatureCard
        icon={<svg data-testid="icon" />}
        title="Keep-Awake Engine"
        description="Monitors idle time."
      />,
    )

    expect(screen.getByTestId('icon')).toBeInTheDocument()
    expect(
      screen.getByRole('heading', { level: 3, name: 'Keep-Awake Engine' }),
    ).toBeInTheDocument()
    expect(screen.getByText('Monitors idle time.')).toBeInTheDocument()
  })

  it('applies an explicitly provided accent', () => {
    const { container } = render(
      <FeatureCard
        icon={<svg />}
        title="Themed"
        description="Blue accent."
        accent="blue"
      />,
    )

    expect(container.querySelector('.bg-accent-blue\\/10')).not.toBeNull()
  })
})
