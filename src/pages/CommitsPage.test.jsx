import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, within, fireEvent, cleanup } from '@testing-library/react'
import CommitsPage from './CommitsPage.jsx'

afterEach(cleanup)

const makeCommits = (count) =>
  Array.from({ length: count }, (_, i) => ({
    hash: `abc123def${i}`,
    subject: `commit ${i}`,
    author_name: 'Alice',
    author_date: '2024-01-01T00:00:00',
    additions: 1,
    deletions: 0,
    files: [{ path: `f${i}.js`, status: 'added' }],
  }))

describe('CommitsPage', () => {
  it('shows the files column count from commit.files', () => {
    render(
      <CommitsPage repository={{ name: 'repo' }} analysisData={{ commits: makeCommits(3) }} loading={false} />,
    )
    // All three rows render a "1" in the files column (files.length).
    const table = screen.getByRole('table')
    expect(within(table).getAllByText('1').length).toBeGreaterThanOrEqual(3)
  })

  it('renders a bounded table with a show-more control', () => {
    render(
      <CommitsPage repository={{ name: 'repo' }} analysisData={{ commits: makeCommits(450) }} loading={false} />,
    )
    const table = screen.getByRole('table')

    // Only the first 200 commits are mounted initially (200 data rows).
    expect(within(table).getAllByRole('row').length - 1).toBe(200)
    expect(screen.getByText(/Showing 200 of 450/)).toBeTruthy()

    fireEvent.click(screen.getByRole('button', { name: /Show more/ }))

    expect(within(table).getAllByRole('row').length - 1).toBe(400)
    expect(screen.getByText(/Showing 400 of 450/)).toBeTruthy()
  })
})