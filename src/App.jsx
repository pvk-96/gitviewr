import React, { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import Sidebar from './components/Sidebar.jsx'
import RepositoryPage from './pages/RepositoryPage.jsx'
import CommitsPage from './pages/CommitsPage.jsx'
import ChangesPage from './pages/ChangesPage.jsx'
import AnalyticsPage from './pages/AnalyticsPage.jsx'
import RecentPage from './pages/RecentPage.jsx'
import ExportPage from './pages/ExportPage.jsx'
import AboutPage from './pages/AboutPage.jsx'
import { useAnalysisProgress } from './hooks/useAnalysisProgress.jsx'

export default function App() {
  const [activeTab, setActiveTab] = useState('repository')
  const [repository, setRepository] = useState(null)
  const [analysisData, setAnalysisData] = useState(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState(null)
  const progressPhase = useAnalysisProgress()

  const handleRepositoryLoaded = useCallback((data) => {
    setRepository(data.repository)
    setAnalysisData(data)
    setError(null)
  }, [])

  const handleError = useCallback((msg) => {
    setError(msg)
  }, [])

  const handleClearError = useCallback(() => {
    setError(null)
  }, [])

  const handleSwitchRepository = useCallback(() => {
    setRepository(null)
    setAnalysisData(null)
    setError(null)
    setActiveTab('repository')
  }, [])

  const handleLoadRecent = useCallback(
    async (recent) => {
      setLoading(true)
      setError(null)
      try {
        if (recent.source_type === 'local') {
          const result = await invoke('analyze_local_repository', { path: recent.path })
          handleRepositoryLoaded(result)
        } else {
          const result = await invoke('analyze_github_repository', { url: recent.path })
          handleRepositoryLoaded(result)
        }
        setActiveTab('repository')
      } catch (err) {
        setError(err.toString() || 'Failed to load repository')
      } finally {
        setLoading(false)
      }
    },
    [handleRepositoryLoaded],
  )

  const renderPage = () => {
    const common = {
      repository,
      analysisData,
      loading,
      error,
      progressPhase,
      onRepositoryLoaded: handleRepositoryLoaded,
      onError: handleError,
      onClearError: handleClearError,
      onSwitchRepository: handleSwitchRepository,
    }

    switch (activeTab) {
      case 'repository':
        return <RepositoryPage {...common} setLoading={setLoading} />
      case 'commits':
        return <CommitsPage {...common} />
      case 'changes':
        return <ChangesPage {...common} />
      case 'analytics':
        return <AnalyticsPage {...common} />
      case 'recent':
        return <RecentPage {...common} onLoadRecent={handleLoadRecent} />
      case 'export':
        return <ExportPage {...common} />
      case 'about':
        return <AboutPage />
    }
  }

  return (
    <div className="app-layout">
      <Sidebar activeTab={activeTab} onTabChange={setActiveTab} repository={repository} />
      <main className="main-content">{renderPage()}</main>
    </div>
  )
}