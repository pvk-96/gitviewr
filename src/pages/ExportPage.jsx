import React, { useState } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import NoRepositoryState from '../components/NoRepositoryState.jsx'

const EXPORT_TYPES = [
  { id: 'html', label: 'HTML Report', description: 'A standalone HTML report', icon: '📄', ext: 'html', filter: 'HTML' },
  { id: 'json', label: 'JSON Data', description: 'Machine-readable analysis data', icon: '🧾', ext: 'json', filter: 'JSON' },
  { id: 'pdf', label: 'PDF Report', description: 'A readable PDF report', icon: '📑', ext: 'pdf', filter: 'PDF' },
]

const slug = (s) => (s || 'repository').replace(/[^a-zA-Z0-9-_]/g, '-')

export default function ExportPage({ repository, analysisData }) {
  const [exporting, setExporting] = useState(null)
  const [result, setResult] = useState(null)
  const [error, setError] = useState(null)

  const handleExport = async (type) => {
    if (!analysisData) {
      setError('No analysis data available to export.')
      return
    }
    setExporting(type.id)
    setResult(null)
    setError(null)
    try {
      const defaultPath = `GitViewr-${slug(repository?.name)}.${type.ext}`
      const target = await save({
        defaultPath,
        filters: [{ name: type.filter, extensions: [type.ext] }],
      })
      if (!target) {
        return
      }
      await invoke('export_report', {
        format: type.id,
        targetPath: target,
        analysisJson: JSON.stringify(analysisData),
      })
      setResult({ type: type.id, path: target })
    } catch (err) {
      setError(err.toString() || 'Export failed.')
    } finally {
      setExporting(null)
    }
  }

  const renderBody = () => {
    if (!repository) {
      return <NoRepositoryState message="Open a repository to export its analysis report." />
    }

    return (
      <>
        {error && <div className="error-banner">{error}</div>}
        {result && (
          <div className="success-banner">
            <strong>
              {EXPORT_TYPES.find((t) => t.id === result.type)?.label}
            </strong>{' '}
            exported to <span className="font-mono">{result.path}</span>
          </div>
        )}

        <div className="export-options">
          {EXPORT_TYPES.map((type) => (
            <div
              key={type.id}
              className="export-card"
              role="button"
              tabIndex={0}
              onClick={() => handleExport(type)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleExport(type)
              }}
            >
              <div className="icon">{type.icon}</div>
              <h4>{type.label}</h4>
              <p>{type.description}</p>
              {exporting === type.id && (
                <div className="mt-2 flex items-center justify-center gap-2">
                  <div className="loading-spinner" style={{ width: 16, height: 16 }} />
                  <span className="text-xs">Exporting…</span>
                </div>
              )}
            </div>
          ))}
        </div>

        <p className="text-sm text-muted">
          The exported {repository.name} analysis includes repository metadata,
          commit history, change statistics and analytics.
        </p>
      </>
    )
  }

  return (
    <div className="tab-content">
      <div className="page-header">
        <h2>Export</h2>
        <p>Export the analysis report for the currently loaded repository.</p>
      </div>
      {renderBody()}
    </div>
  )
}