import { describe, it, expect } from 'vitest'
import { formatNumber, shortHash, formatRelativeTime, formatDate } from './format.js'

describe('formatNumber', () => {
  it('formats small numbers', () => {
    expect(formatNumber(42)).toBe('42')
  })

  it('formats thousands with separators', () => {
    expect(formatNumber(12345)).toBe('12,345')
  })

  it('handles null/undefined', () => {
    expect(formatNumber(null)).toBe('—')
    expect(formatNumber(undefined)).toBe('—')
  })

  it('compacts very large numbers', () => {
    expect(formatNumber(1234567)).toMatch(/1\.2M|1,234,567/)
  })
})

describe('shortHash', () => {
  it('truncates to 7 chars', () => {
    expect(shortHash('0123456789abcdef')).toBe('0123456')
  })

  it('returns dash for empty', () => {
    expect(shortHash('')).toBe('—')
    expect(shortHash(null)).toBe('—')
  })
})

describe('formatRelativeTime', () => {
  const now = Date.now()
  it('minutes ago', () => {
    expect(formatRelativeTime(new Date(now - 5 * 60 * 1000).toISOString())).toBe(
      '5 minutes ago',
    )
  })
  it('days ago', () => {
    expect(formatRelativeTime(new Date(now - 2 * 86400 * 1000).toISOString())).toBe(
      '2 days ago',
    )
  })
  it('handles missing', () => {
    expect(formatRelativeTime(null)).toBe('—')
  })
})

describe('formatDate', () => {
  it('formats ISO date', () => {
    const out = formatDate('2021-06-01T12:00:00')
    expect(out).toContain('2021')
    expect(out).toContain('Jun')
  })
  it('handles invalid', () => {
    expect(formatDate('nonsense')).toBe('nonsense')
  })
})