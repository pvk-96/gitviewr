import React, { useState, useMemo } from 'react'
import NoRepositoryState from '../components/NoRepositoryState.jsx'
import CommitDetailModal from '../components/CommitDetailModal.jsx'
import { formatDate, shortHash } from '../utils/format.js'

const PAGE_SIZE = 200

export default function CommitsPage({ repository, analysisData, loading }) {
  const [search, setSearch] = useState('')
  const [selectedCommit, setSelectedCommit] = useState(null)
  const [visibleCount, setVisibleCount] = useState(PAGE_SIZE)

  const commits = analysisData?.commits || []

  const filteredCommits = useMemo(() => {
    if (!search.trim()) return commits
    const q = search.toLowerCase()
    return commits.filter((c) => {
      return (
        (c.hash && c.hash.toLowerCase().includes(q)) ||
        (c.subject && c.subject.toLowerCase().includes(q)) ||
        (c.author_name && c.author_name.toLowerCase().includes(q)) ||
        (c.author_email && c.author_email.toLowerCase().includes(q))
      )
    })
  }, [commits, search])

  const visibleCommits = filteredCommits.slice(0, visibleCount)

  const renderBody = () => {
    if (loading) {
      return (
        <div className="loading-state">
          <div className="loading-spinner" />
          <p>Loading commit history…</p>
        </div>
      )
    }

    if (!repository) {
      return (
        <NoRepositoryState message="Open a repository to view its commit history." />
      )
    }

    if (commits.length === 0) {
      return (
        <div className="empty-state">
          <h3>No commits found</h3>
          <p>This repository does not appear to have any commits.</p>
        </div>
      )
    }

    return (
      <>
        <div className="search-bar">
          <input
            className="search-input"
            type="text"
            placeholder="Search by hash, message, or author…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <span className="text-sm text-muted" style={{ alignSelf: 'center' }}>
            {filteredCommits.length} of {commits.length}
          </span>
        </div>

        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th style={{ width: 80 }}>Hash</th>
                <th>Message</th>
                <th>Author</th>
                <th>Date</th>
                <th>Files</th>
                <th>+</th>
                <th>−</th>
              </tr>
            </thead>
            <tbody>
              {visibleCommits.map((commit) => (
                <tr
                  key={commit.hash}
                  style={{ cursor: 'pointer' }}
                  onClick={() => setSelectedCommit(commit)}
                >
                  <td className="font-mono" style={{ color: 'var(--accent)' }}>
                    {shortHash(commit.hash)}
                  </td>
                  <td style={{ color: 'var(--text-primary)' }}>
                    {commit.subject}
                  </td>
                  <td>{commit.author_name}</td>
                  <td>{formatDate(commit.author_date)}</td>
                  <td>{commit.files.length}</td>
                  <td style={{ color: 'var(--success)' }}>
                    {commit.additions}
                  </td>
                  <td style={{ color: 'var(--danger)' }}>
                    {commit.deletions}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {filteredCommits.length > visibleCount && (
          <div className="flex items-center gap-2 mt-2">
            <button
              className="btn btn-secondary"
              onClick={() => setVisibleCount((n) => n + PAGE_SIZE)}
            >
              Show more
            </button>
            <span className="text-sm text-muted">
              Showing {visibleCount} of {filteredCommits.length} commits
            </span>
          </div>
        )}

        <CommitDetailModal
          commit={selectedCommit}
          onClose={() => setSelectedCommit(null)}
        />
      </>
    )
  }

  return (
    <div className="tab-content">
      <div className="page-header">
        <h2>Commits</h2>
        <p>Browse the repository commit history.</p>
      </div>
      {renderBody()}
    </div>
  )
}