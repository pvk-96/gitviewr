import { describe, it, expect, afterEach } from 'vitest'
import { render, screen, within, fireEvent, cleanup } from '@testing-library/react'
import ChangesPage from './ChangesPage.jsx'

afterEach(cleanup)

const analysisData = {
  file_stats: [
    { path: 'a.js', status: 'added', additions: 5, deletions: 0, change_count: 1, last_committed: '2024-01-01' },
    { path: 'b.js', status: 'modified', additions: 1, deletions: 1, change_count: 2, last_committed: '2024-01-02' },
    { path: 'c.js', status: 'deleted', additions: 0, deletions: 3, change_count: 1, last_committed: '2024-01-03' },
    { path: 'd.js', status: 'renamed', additions: 0, deletions: 0, change_count: 1, last_committed: '2024-01-04' },
  ],
}

describe('ChangesPage', () => {
  it('renders one file entry per status and trusts the backend status field', () => {
    render(
      <ChangesPage repository={{ name: 'repo' }} analysisData={analysisData} loading={false} />,
    )

    const table = screen.getByRole('table')
    for (const status of ['added', 'modified', 'deleted', 'renamed']) {
      expect(
        within(table).getAllByText(status).length,
        `exactly one ${status} row in the table`,
      ).toBe(1)
    }
  })

  it('summaries match the backend statuses, not size heuristics', () => {
    render(
      <ChangesPage repository={{ name: 'repo' }} analysisData={analysisData} loading={false} />,
    )

    const grid = document.querySelector('.stat-grid')
    expect(grid).toBeTruthy()
    for (const label of ['Added', 'Modified', 'Deleted', 'Renamed']) {
      const statItem = within(grid).getByText(label).closest('.stat-item')
      expect(statItem.querySelector('.stat-value').textContent).toBe('1')
    }
  })

  it('filters rows by status using the backend status field', () => {
    render(
      <ChangesPage repository={{ name: 'repo' }} analysisData={analysisData} loading={false} />,
    )

    const addedButton = within(screen.getByRole('tablist')).getByRole('button', { name: 'Added' })
    fireEvent.click(addedButton)

    expect(screen.getByText('a.js')).toBeTruthy()
    expect(screen.queryByText('b.js')).toBeNull()
    expect(screen.queryByText('c.js')).toBeNull()
    expect(screen.queryByText('d.js')).toBeNull()
  })
})