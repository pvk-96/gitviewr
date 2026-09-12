import React from 'react'
import Icons from '../utils/icons.jsx'

const NAV_ITEMS = [
  { id: 'repository', label: 'Repository', icon: Icons.repository },
  { id: 'commits', label: 'Commits', icon: Icons.commits },
  { id: 'changes', label: 'Changes', icon: Icons.changes },
  { id: 'analytics', label: 'Analytics', icon: Icons.analytics },
  { id: 'recent', label: 'Recent', icon: Icons.recent },
  { id: 'export', label: 'Export', icon: Icons.export },
  { id: 'about', label: 'About', icon: Icons.about },
]

export default function Sidebar({ activeTab, onTabChange, repository }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-brand">
        <h1>
          <span style={{ width: 22, height: 22 }}>{Icons.gitBranch}</span>
          GitViewr
        </h1>
        <div className="tagline">Understand your Git repositories.</div>
      </div>

      <nav className="sidebar-nav">
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            className={`nav-item ${activeTab === item.id ? 'active' : ''}`}
            onClick={() => onTabChange(item.id)}
          >
            {item.icon}
            <span>{item.label}</span>
          </button>
        ))}
      </nav>

      {repository && (
        <div className="sidebar-footer">
          <div className="repo-info">
            <span className="repo-name">{repository.name}</span>
          </div>
          <div className="repo-info">{repository.path}</div>
        </div>
      )}
    </aside>
  )
}