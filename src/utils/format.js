function formatDateTime(isoString, withTime) {
  if (!isoString) return '—'
  const date = new Date(isoString)
  if (isNaN(date.getTime())) return isoString
  if (withTime) {
    return date.toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  }
  return date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: '2-digit',
  })
}

export function formatDate(isoString) {
  return formatDateTime(isoString, true)
}

export function formatShortDate(isoString) {
  return formatDateTime(isoString, false)
}

export function formatRelativeTime(isoString) {
  if (!isoString) return '—'
  const date = new Date(isoString)
  if (isNaN(date.getTime())) return isoString
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000)
  const intervals = [
    { key: 'year', seconds: 31536000 },
    { key: 'month', seconds: 2592000 },
    { key: 'day', seconds: 86400 },
    { key: 'hour', seconds: 3600 },
    { key: 'minute', seconds: 60 },
  ]
  for (const { key, seconds: s } of intervals) {
    const count = Math.floor(seconds / s)
    if (count >= 1) {
      return `${count} ${key}${count > 1 ? 's' : ''} ago`
    }
  }
  return 'just now'
}

export function formatNumber(num) {
  if (num === null || num === undefined) return '—'
  return new Intl.NumberFormat('en-US', {
    notation: num >= 100000 ? 'compact' : 'standard',
    maximumFractionDigits: 1,
  }).format(num)
}

export function shortHash(hash) {
  if (!hash) return '—'
  return hash.slice(0, 7)
}