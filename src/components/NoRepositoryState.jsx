import React from 'react'
import Icons from '../utils/icons.jsx'

export default function NoRepositoryState({ title = 'No repository loaded', message }) {
  return (
    <div className="empty-state">
      {Icons.repository}
      <h3>{title}</h3>
      <p>
        {message || (
          <>Open a repository from the Repository tab to view this information.</>
        )}
      </p>
    </div>
  )
}