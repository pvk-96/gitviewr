import React, { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import Icons from '../utils/icons.jsx'

const DONATION_URL = {
  url: 'https://www.buymeacoffee.com/pvk96',
  label: 'Buy Me a Coffee',
}

export default function AboutPage() {
  const [version, setVersion] = useState(null)

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {})
  }, [])

  const openExternal = (url) => {
    invoke('open_external_url', { url }).catch((err) => {
      console.error('Failed to open external URL:', err)
    })
  }

  const openDonation = () => {
    openExternal(DONATION_URL.url)
  }

  const openWebsite = () => {
    openExternal('https://pvk96.in')
  }

  return (
    <div className="about-page">
      <h2>GitViewr</h2>
      <p className="tagline">Understand your Git repositories.</p>

      <div className="about-section">
        <h3>About</h3>
        <div className="flex items-center gap-3" style={{ alignItems: 'flex-start' }}>
          <div style={{ width: 56, height: 56 }}>
            {Icons.gitBranch}
          </div>
          <div>
            <p className="text-sm text-secondary">
              GitViewr is a desktop application for understanding and analyzing
              Git repositories. It inspects repository metadata, commit history
              and change statistics to help you understand how a project has
              evolved.
            </p>
            <p className="text-sm text-secondary mt-2">
              GitViewr is not a Git client. It does not commit, push, pull,
              merge, or create branches.
            </p>
          </div>
        </div>
      </div>

      <div className="about-section">
        <h3>Details</h3>
        <div className="about-item">
          <span className="label">Version</span>
          <span className="value">{version || '0.1.0'}</span>
        </div>
        <div className="about-item">
          <span className="label">Application ID</span>
          <span className="value font-mono">in.pvk96.gitviewr</span>
        </div>
        <div className="about-item">
          <span className="label">Tech Stack</span>
          <span className="value">Tauri 2 · Rust · React</span>
        </div>
        <div className="about-item">
          <span className="label">Project</span>
          <span className="value">
            <a
              href="#"
              onClick={(e) => {
                e.preventDefault()
                openWebsite()
              }}
            >
              pvk96.in
            </a>
          </span>
        </div>
        <div className="about-item">
          <span className="label">Developed by</span>
          <span className="value">PVK</span>
        </div>
      </div>

      <div className="about-section">
        <h3>Check out my other projects</h3>
        <button
          className="btn btn-secondary"
          onClick={openWebsite}
        >
          {Icons.external}
          <span>pvk96.in</span>
        </button>
      </div>

      <div className="donation-section">
        <p>Enjoying GitViewr?</p>
        <button className="btn btn-primary" onClick={openDonation}>
          {Icons.coffee}
          <span>Support my work — {DONATION_URL.label}</span>
        </button>
      </div>
    </div>
  )
}