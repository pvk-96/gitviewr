import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import Icons from '../utils/icons.jsx'
import { formatNumber, formatDate } from '../utils/format.js'

export default function RepositoryPage({
  repository,
  analysisData,
  loading,
  error,
  progressPhase,
  setLoading,
  onRepositoryLoaded,
  onError,
  onClearError,
  onSwitchRepository,
}) {
  const [githubUrl, setGithubUrl] = useState('')

  const handleOpenLocal = async () => {
    try {
      const selected = await open({ directory: true, multiple: false })
      if (!selected) return
      setLoading(true)
      onClearError()
      try {
        const result = await invoke('analyze_local_repository', { path: selected })
        onRepositoryLoaded(result)
      } catch (err) {
        onError(err.toString() || 'Unable to open repository.')
      } finally {
        setLoading(false)
      }
    } catch (err) {
      onError('Unable to open the folder picker: ' + err)
    }
  }

  const handleOpenGithub = async () => {
    const url = githubUrl.trim()
    if (!url) {
      onError('Please enter a GitHub repository URL.')
      return
    }
    setLoading(true)
    onClearError()
    try {
      const result = await invoke('analyze_github_repository', { url })
      onRepositoryLoaded(result)
      setGithubUrl('')
    } catch (err) {
      onError(err.toString() || 'Unable to clone repository.')
    } finally {
      setLoading(false)
    }
  }

  const handleKeyDown = (e) => {
    if (e.key === 'Enter') {
      handleOpenGithub()
    }
  }

  const renderSourceCard = (title, description, icon, action) => (
    <div className="card">
      <div className="card-header">
        <h3>{title}</h3>
        <span style={{ width: 24, height: 24, color: 'var(--text-muted)' }}>{icon}</span>
      </div>
      <p className="text-sm text-secondary">{description}</p>
      {action}
    </div>
  )

  const getErrorMessage = (raw) => {
    try {
      const parsed = JSON.parse(raw)
      if (parsed && parsed.message) return parsed.message
      return raw
    } catch {
      return raw
    }
  }

  const renderBody = () => {
    if (loading) {
      return (
        <div className="loading-state">
          <div className="loading-spinner" />
          <p>{progressPhase || 'Analyzing repository…'}</p>
        </div>
      )
    }

    if (!repository) {
      return (
        <div className="flex gap-4">
          <div style={{ flex: 1, minWidth: 0 }}>
            {renderSourceCard(
              'Open Local Repository',
              'Select an existing Git repository folder on your computer.',
              Icons.folder,
              <div className="btn-group">
                <button className="btn btn-primary" onClick={handleOpenLocal}>
                  Open Local Repository
                </button>
              </div>,
            )}
          </div>

          <div style={{ flex: 1, minWidth: 0 }}>
            {renderSourceCard(
              'GitHub Repository URL',
              'Analyze a public GitHub repository. It will be cloned to a managed location.',
              Icons.github,
              <div className="input-group mt-2">
                <input
                  className="text-input"
                  type="text"
                  placeholder="https://github.com/owner/repository"
                  value={githubUrl}
                  onChange={(e) => setGithubUrl(e.target.value)}
                  onKeyDown={handleKeyDown}
                />
                <button className="btn btn-primary" onClick={handleOpenGithub}>
                  Analyze
                </button>
              </div>,
            )}
          </div>
        </div>
      )
    }

    const meta = analysisData?.metadata || {}
    const stats = analysisData?.stats || {}

    const statItems = [
      { label: 'Current Branch', value: stats.branch || '—' },
      { label: 'Commits', value: formatNumber(stats.commit_count) },
      { label: 'Branches', value: formatNumber(stats.branch_count) },
      { label: 'Tracked Files', value: formatNumber(stats.tracked_files) },
      { label: 'Contributors', value: formatNumber(stats.contributor_count) },
      { label: 'First Commit', value: formatDate(stats.first_commit_date) },
      { label: 'Latest Commit', value: formatDate(stats.latest_commit_date) },
      { label: 'Additions', value: formatNumber(stats.total_additions) },
      { label: 'Deletions', value: formatNumber(stats.total_deletions) },
      { label: 'Net Change', value: formatNumber(stats.net_change) },
    ]

    return (
      <>
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 style={{ fontSize: 22, fontWeight: 600 }}>{repository.name}</h2>
            <p className="text-sm text-muted font-mono">{repository.path}</p>
          </div>
          <div className="flex gap-2 items-center">
            <span className="badge badge-added">
              {repository.source_type === 'local' ? 'Local' : 'GitHub'}
            </span>
            <button className="btn btn-secondary" onClick={onSwitchRepository}>
              Switch Repository
            </button>
          </div>
        </div>

        <div className="stat-grid">
          {statItems.map((item) => (
            <div className="stat-item" key={item.label}>
              <div className="stat-label">{item.label}</div>
              <div className="stat-value small">{item.value}</div>
            </div>
          ))}
        </div>

        <div className="card">
          <div className="card-header">
            <h3>Repository Information</h3>
          </div>
          <table>
            <tbody>
              <tr>
                <td style={{ width: 200, color: 'var(--text-muted)' }}>Name</td>
                <td>{repository.name}</td>
              </tr>
              <tr>
                <td style={{ width: 200, color: 'var(--text-muted)' }}>
                  {repository.source_type === 'github' ? 'URL' : 'Path'}
                </td>
                <td className="font-mono">
                  {repository.source_type === 'github'
                    ? repository.path
                    : meta.path || repository.path}
                </td>
              </tr>
              <tr>
                <td>Source Type</td>
                <td>{repository.source_type === 'local' ? 'Local' : 'GitHub'}</td>
              </tr>
              <tr>
                <td>Default Branch</td>
                <td>{meta.default_branch || '—'}</td>
              </tr>
              {meta.remote_url && (
                <tr>
                  <td>Remote</td>
                  <td className="font-mono">{meta.remote_url}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>

        {meta.contributors && meta.contributors.length > 0 && (
          <div className="card">
            <div className="card-header">
              <h3>Contributors</h3>
              <span className="text-sm text-muted">
                {meta.contributors.length} total
              </span>
            </div>
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Commits</th>
                </tr>
              </thead>
              <tbody>
                {meta.contributors.slice(0, 12).map((contributor) => (
                  <tr key={contributor.email || contributor.name}>
                    <td>{contributor.name}</td>
                    <td>{formatNumber(contributor.commits)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </>
    )
  }

  return (
    <div className="tab-content">
      <div className="page-header">
        <h2>Repository</h2>
        <p>Open a repository to begin analysis.</p>
      </div>

      {error && (
        <div className="error-banner">
          <div className="flex items-center justify-between">
            <pre style={{ whiteSpace: 'pre-wrap', fontFamily: 'inherit' }}>
              {getErrorMessage(error)}
            </pre>
            {onClearError && (
              <button
                className="btn btn-secondary"
                onClick={onClearError}
                style={{ padding: '2px 8px', fontSize: 12 }}
                title="Dismiss"
              >
                {Icons.x}
              </button>
            )}
          </div>
        </div>
      )}

      {renderBody()}
    </div>
  )
}