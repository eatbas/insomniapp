import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SectionHeading } from './SectionHeading'

describe('SectionHeading', () => {
  it('renders the badge, heading, and description', () => {
    render(
      <SectionHeading
        badge="Features"
        title="Everything You Need"
        description="A short description."
      />,
    )

    expect(screen.getByText('Features')).toBeInTheDocument()
    expect(
      screen.getByRole('heading', { level: 2, name: 'Everything You Need' }),
    ).toBeInTheDocument()
    expect(screen.getByText('A short description.')).toBeInTheDocument()
  })
})
