import React, { useMemo } from 'react'
import {
  ResponsiveContainer,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip,
  CartesianGrid,
  LineChart,
  Line,
} from 'recharts'
import NoRepositoryState from '../components/NoRepositoryState.jsx'
import { formatNumber, formatShortDate } from '../utils/format.js'

const COLORS = {
  additions: '#51cf66',
  deletions: '#ff6b6b',
  net: '#fcc419',
  commits: '#4dabf7',
}

function ChartCard({ title, subtitle, children }) {
  return (
    <div className="chart-container">
      <div className="mb-3">
        <h3 style={{ marginBottom: 2 }}>{title}</h3>
        {subtitle && <p className="text-xs text-muted">{subtitle}</p>}
      </div>
      {children}
    </div>
  )
}

export default function AnalyticsPage({ repository, analysisData, loading }) {
  const analytics = analysisData?.analytics || {}

  const contributorData = useMemo(() => {
    const rows = Object.values(analytics.contributors || {}).sort(
      (a, b) => b.commits - a.commits,
    )
    return rows.slice(0, 10).map((c) => ({
      name: c.name || c.email || 'Unknown',
      commits: c.commits,
      additions: c.additions,
      deletions: c.deletions,
    }))
  }, [analytics.contributors])

  const commitsOverTime = useMemo(() => {
    const byMonth = (analytics.commits_over_time || []).map((item) => ({
      period: item.period,
      commits: item.commits,
    }))
    return byMonth.slice(-24)
  }, [analytics.commits_over_time])

  const churnOverTime = useMemo(() => {
    const row = (analytics.churn_over_time || []).map((item) => ({
      period: item.period,
      additions: item.additions,
      deletions: item.deletions,
    }))
    return row.slice(-24)
  }, [analytics.churn_over_time])

  const fileHotspots = useMemo(() => {
    const rows = Object.values(analytics.file_hotspots || {}).sort(
      (a, b) => b.changes - a.changes,
    )
    return rows.slice(0, 10)
  }, [analytics.file_hotspots])

  const growthTimeline = useMemo(() => {
    const rows = (analytics.timeline || []).map((item) => ({
      period: item.period,
      net: item.net,
    }))
    return rows.slice(-24)
  }, [analytics.timeline])

  const renderBody = () => {
    if (loading) {
      return (
        <div className="loading-state">
          <div className="loading-spinner" />
          <p>Computing analytics…</p>
        </div>
      )
    }

    if (!repository) {
      return (
        <NoRepositoryState message="Open a repository to view its analytics." />
      )
    }

    if (contributorData.length === 0 && commitsOverTime.length === 0) {
      return (
        <div className="empty-state">
          <h3>Not enough data</h3>
          <p>This repository does not have enough history for analytics yet.</p>
        </div>
      )
    }

    return (
      <>
        <div className="stat-grid">
          <div className="stat-item">
            <div className="stat-label">Total Commits</div>
            <div className="stat-value small">
              {formatNumber(analytics.total_commits)}
            </div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Total Additions</div>
            <div className="stat-value small" style={{ color: 'var(--success)' }}>
              {formatNumber(analytics.total_additions)}
            </div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Total Deletions</div>
            <div className="stat-value small" style={{ color: 'var(--danger)' }}>
              {formatNumber(analytics.total_deletions)}
            </div>
          </div>
          <div className="stat-item">
            <div className="stat-label">Net Change</div>
            <div className="stat-value small">
              {formatNumber(analytics.net_change)}
            </div>
          </div>
        </div>

        {commitsOverTime.length > 0 && (
          <ChartCard title="Commits over time" subtitle="Commit frequency by month">
            <ResponsiveContainer width="100%" height={240}>
              <BarChart data={commitsOverTime}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" />
                <XAxis dataKey="period" stroke="var(--text-muted)" fontSize={11} />
                <YAxis stroke="var(--text-muted)" fontSize={11} allowDecimals={false} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                  labelStyle={{ color: 'var(--text-secondary)' }}
                />
                <Bar dataKey="commits" fill={COLORS.commits} radius={[3, 3, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {churnOverTime.length > 0 && (
          <ChartCard
            title="Code churn"
            subtitle="Additions and deletions per month"
          >
            <ResponsiveContainer width="100%" height={240}>
              <BarChart data={churnOverTime}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" />
                <XAxis dataKey="period" stroke="var(--text-muted)" fontSize={11} />
                <YAxis stroke="var(--text-muted)" fontSize={11} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                  labelStyle={{ color: 'var(--text-secondary)' }}
                />
                <Bar dataKey="additions" stackId="a" fill={COLORS.additions} radius={[3, 3, 0, 0]} />
                <Bar dataKey="deletions" stackId="a" fill={COLORS.deletions} radius={[3, 3, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {contributorData.length > 0 && (
          <ChartCard
            title="Contributor activity"
            subtitle="Commits by contributor"
          >
            <ResponsiveContainer width="100%" height={240}>
              <BarChart data={contributorData} layout="vertical" margin={{ left: 40 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" />
                <XAxis type="number" stroke="var(--text-muted)" fontSize={11} allowDecimals={false} />
                <YAxis
                  type="category"
                  dataKey="name"
                  width={140}
                  stroke="var(--text-muted)"
                  fontSize={11}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                />
                <Bar dataKey="commits" fill={COLORS.commits} radius={[0, 3, 3, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {contributorData.length > 0 && (
          <ChartCard
            title="Contributions per contributor"
            subtitle="Lines added and removed by top contributors"
          >
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={contributorData} layout="vertical" margin={{ left: 40 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" />
                <XAxis type="number" stroke="var(--text-muted)" fontSize={11} />
                <YAxis
                  type="category"
                  dataKey="name"
                  width={140}
                  stroke="var(--text-muted)"
                  fontSize={11}
                />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                />
                <Bar dataKey="additions" stackId="a" fill={COLORS.additions} radius={[3, 3, 0, 0]} />
                <Bar dataKey="deletions" stackId="a" fill={COLORS.deletions} radius={[3, 3, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {growthTimeline.length > 0 && (
          <ChartCard
            title="Cumulative code growth"
            subtitle="Net lines in the repository by month"
          >
            <ResponsiveContainer width="100%" height={240}>
              <LineChart data={growthTimeline}>
                <CartesianGrid strokeDasharray="3 3" stroke="var(--border-color)" />
                <XAxis dataKey="period" stroke="var(--text-muted)" fontSize={11} />
                <YAxis stroke="var(--text-muted)" fontSize={11} allowDecimals={false} />
                <Tooltip
                  contentStyle={{
                    backgroundColor: 'var(--bg-tertiary)',
                    border: '1px solid var(--border-color)',
                    borderRadius: 6,
                    fontSize: 12,
                  }}
                />
                <Line type="monotone" dataKey="net" stroke={COLORS.net} strokeWidth={2} dot={false} />
              </LineChart>
            </ResponsiveContainer>
          </ChartCard>
        )}

        {fileHotspots.length > 0 && (
          <ChartCard
            title="File hotspots"
            subtitle="Files changed most frequently"
          >
            <div className="table-container">
              <table>
                <thead>
                  <tr>
                    <th>File</th>
                    <th>Changes</th>
                    <th>+</th>
                    <th>−</th>
                    <th>Last changed</th>
                  </tr>
                </thead>
                <tbody>
                  {fileHotspots.map((file) => (
                    <tr key={file.path}>
                      <td className="font-mono" style={{ fontSize: 12 }}>
                        {file.path}
                      </td>
                      <td>{file.changes}</td>
                      <td style={{ color: 'var(--success)' }}>
                        {formatNumber(file.additions)}
                      </td>
                      <td style={{ color: 'var(--danger)' }}>
                        {formatNumber(file.deletions)}
                      </td>
                      <td>{formatShortDate(file.last_changed)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </ChartCard>
        )}
      </>
    )
  }

  return (
    <div className="tab-content">
      <div className="page-header">
        <h2>Analytics</h2>
        <p>Repository-level statistics and visualization.</p>
      </div>
      {renderBody()}
    </div>
  )
}