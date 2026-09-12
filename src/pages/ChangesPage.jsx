import React, { useState, useMemo } from 'react'
import NoRepositoryState from '../components/NoRepositoryState.jsx'
import { formatNumber } from '../utils/format.js'

const STATUS_COLORS = {
  added: 'badge-added',
  modified: 'badge-modified',
  deleted: 'badge-deleted',
  renamed: 'badge-renamed',
}

const PAGE_SIZE = 200

export default function ChangesPage({ repository, analysisData, loading }) {
  const [statusFilter, setStatusFilter] = useState('all')
  const [search, setSearch] = useState('')

  const fileStats = analysisData?.file_stats || []

  const summary = useMemo(() => {
    const counts = { added: 0, modified: 0, deleted: 0, renamed: 0 }
    for (const file of fileStats) {
      counts[file.status] += 1
    }
    return counts
  }, [fileStats])

  const filteredFiles = useMemo(() => {
    let list = fileStats
    if (statusFilter !== 'all') {
      list = list.filter((f) => f.status === statusFilter)
    }
    if (search.trim()) {
      const q = search.toLowerCase()
      list = list.filter((f) => f.path.toLowerCase().includes(q))
    }
    return list
  }, [fileStats, statusFilter, search])

  const renderBody = () => {
    if (loading) {
      return (
        <div className="loading-state">
          <div className="loading-spinner" />
          <p>Loading changes…</p>
        </div>
      )
    }

    if (!repository) {
      return (
        <NoRepositoryState message="Open a repository to view its file changes." />
      )
    }

    if (fileStats.length === 0) {
      return (
        <div className="empty-state">
          <h3>No change data available</h3>
          <p>There is not enough commit history for this repository yet.</p>
        </div>
      )
    }

    return (
      <>
        <div className="stat-grid">
          <div className="stat-item">
            <div className="stat-label">Total Files</div>
            <div className="stat-value small">{formatNumber(fileStats.length)}</div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Added</div>
            <div className="stat-value small" style={{ color: 'var(--success)' }}>
              {formatNumber(summary.added)}
            </div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Modified</div>
            <div className="stat-value small" style={{ color: 'var(--warning)' }}>
              {formatNumber(summary.modified)}
            </div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Deleted</div>
            <div className="stat-value small" style={{ color: 'var(--danger)' }}>
              {formatNumber(summary.deleted)}
            </div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Renamed</div>
            <div className="stat-value small" style={{ color: 'var(--info)' }}>
              {formatNumber(summary.renamed)}
            </div>
          </div>
        </div>

        <div className="flex gap-2 mb-3" role="tablist">
          {['all', 'added', 'modified', 'deleted', 'renamed'].map((status) => (
            <button
              key={status}
              className={`btn btn-secondary ${statusFilter === status ? 'btn-primary' : ''}`}
              onClick={() => setStatusFilter(status)}
              style={{ padding: '4px 12px', fontSize: 12 }}
            >
              {status.charAt(0).toUpperCase() + status.slice(1)}
            </button>
          ))}
        </div>

        <div className="search-bar">
          <input
            className="search-input"
            type="text"
            placeholder="Search files…"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
        </div>

        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>File</th>
                <th>Status</th>
                <th>+</th>
                <th>−</th>
                <th>Net</th>
                <th>Changes</th>
              </tr>
            </thead>
            <tbody>
              {filteredFiles.slice(0, PAGE_SIZE).map((file) => (
                <tr key={file.path}>
                  <td className="font-mono" style={{ fontSize: 12 }}>
                    {file.path}
                  </td>
                  <td>
                    <span className={`badge ${STATUS_COLORS[file.status]}`}>
                      {file.status}
                    </span>
                  </td>
                  <td style={{ color: 'var(--success)' }}>
                    {formatNumber(file.additions)}
                  </td>
                  <td style={{ color: 'var(--danger)' }}>
                    {formatNumber(file.deletions)}
                  </td>
                  <td
                    style={{
                      color:
                        file.additions - file.deletions > 0
                          ? 'var(--success)'
                          : file.additions - file.deletions < 0
                            ? 'var(--danger)'
                            : 'var(--text-muted)',
                    }}
                  >
                    {file.additions - file.deletions}
                  </td>
                  <td>{file.change_count}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {filteredFiles.length > PAGE_SIZE && (
          <p className="text-sm text-muted mt-2">
            Showing first {PAGE_SIZE} of {filteredFiles.length} files.
          </p>
        )}
      </>
    )
  }

  return (
    <div className="tab-content">
      <div className="page-header">
        <h2>Changes</h2>
        <p>Understand what changed in this repository.</p>
      </div>
      {renderBody()}
    </div>
  )
}