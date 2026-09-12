import React from 'react'

export default function CommitDetailModal({ commit, onClose }) {
  if (!commit) return null

  const changedFiles = commit.files || []

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal" onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center justify-between mb-3">
          <h3 style={{ marginBottom: 0 }}>{commit.subject || 'Commit'}</h3>
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
        </div>

        <div className="mb-4">
          {commit.body && (
            <p className="text-sm text-secondary" style={{ marginBottom: 12 }}>
              {commit.body}
            </p>
          )}
          <div className="flex gap-4 text-xs">
            <span className="font-mono" style={{ color: 'var(--accent)' }}>
              {commit.hash}
            </span>
          </div>
          <div className="flex gap-4 mt-2 text-xs text-secondary">
            <span>
              <strong>Author:</strong> {commit.author_name || '—'}
            </span>
            <span>
              <strong>Date:</strong> {commit.author_date || '—'}
            </span>
          </div>
        </div>

        <div className="flex gap-2 mb-3">
          <span className="badge badge-added">
            +{commit.additions} additions
          </span>
          <span className="badge badge-deleted">
            −{commit.deletions} deletions
          </span>
          <span className="badge badge-modified">
            {changedFiles.length} file{changedFiles.length === 1 ? '' : 's'}
          </span>
        </div>

        {changedFiles.length > 0 && (
          <div className="table-container">
            <table>
              <thead>
                <tr>
                  <th>File</th>
                  <th>Status</th>
                  <th>+</th>
                  <th>−</th>
                </tr>
              </thead>
              <tbody>
                {changedFiles.map((file) => (
                  <tr key={file.path}>
                    <td className="font-mono" style={{ fontSize: 12 }}>
                      {file.path}
                    </td>
                    <td>
                      <span className={`badge badge-${file.status}`}>
                        {file.status}
                      </span>
                    </td>
                    <td style={{ color: 'var(--success)' }}>
                      {file.additions}
                    </td>
                    <td style={{ color: 'var(--danger)' }}>
                      {file.deletions}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}