import React from 'react'
import { useState, useEffect } from 'react'
import { listen } from '@tauri-apps/api/event'

export function useAnalysisProgress() {
  const [phase, setPhase] = useState('Analyzing repository…')

  useEffect(() => {
    const subscription = listen('analysis-progress', (event) => {
      if (event.payload && event.payload.phase) {
        setPhase(event.payload.phase)
      }
    })

    return () => {
      subscription.then((unlisten) => unlisten()).catch(() => {})
    }
  }, [])

  return phase
}