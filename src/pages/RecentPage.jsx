import React, { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import Icons from '../utils/icons.jsx'
import { formatRelativeTime } from '../utils/format.js'

export default function RecentPage({ onLoadRecent, loading }) {
  const [recentRepos, setRecentRepos] = useState([])
  const [error, setError] = useState(null)

  const loadRecent = useCallback(async () => {
    try {
      const result = await invoke('get_recent_repositories')
      setRecentRepos(result || [])
    } catch (err) {
      setError(err.toString() || 'Failed to load recent repositories.')
    }
  }, [])

  useEffect(() => {
    loadRecent()
  }, [loadRecent])

  const renderBody = () => {
    if (loading) {
      return (
        <div className="loading-state">
          <div className="loading-spinner" />
          <p>Opening repository…</p>
        </div>
      )
    }

    if (error) {
      return <div className="error-banner">{error}</div>
    }

    if (recentRepos.length === 0) {
      return (
        <div className="empty-state">
          {Icons.recent}
          <h3>No recent repositories</h3>
          <p>
            Repositories you open will appear here so you can reopen them
            quickly.
          </p>
        </div>
      )
    }

    return (
      <div className="recent-list">
        {recentRepos.map((repo) => (
          <div
            className="recent-item"
            key={repo.path}
            onClick={() => onLoadRecent(repo)}
            role="button"
            tabIndex={0}
            onKeyDown={(e) => {
              if (e.key === 'Enter') onLoadRecent(repo)
            }}
            title={`Open ${repo.name}`}
          >
            <div className="repo-info">
              <h4>{repo.name}</h4>
              <p className="font-mono" style={{ fontSize: 11 }}>
                {repo.path}
              </p>
            </div>
            <div className="repo-meta">
              <div>
                <span className="type-badge">
                  {repo.source_type === 'local' ? 'Local' : 'GitHub'}
                </span>
              </div>
              <div className="timestamp">Opened {formatRelativeTime(repo.last_opened)}</div>
            </div>
          </div>
        ))}
      </div>
    )
  }

  return (
    <div className="tab-content">
      <div className="page-header">
        <h2>Recent Repositories</h2>
        <p>Quickly reopen repositories you have recently analyzed.</p>
      </div>
      {renderBody()}
    </div>
  )
}