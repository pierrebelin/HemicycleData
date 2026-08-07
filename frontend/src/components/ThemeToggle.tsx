import { useEffect, useState } from 'react'

type Theme = 'light' | 'dark'

const CLE = 'theme'

/**
 * Le script de `index.html` a déjà posé l'attribut avant le premier rendu.
 * On le relit plutôt que de recalculer la préférence, pour que le bouton
 * affiche exactement ce que la page montre.
 */
function themeCourant(): Theme {
  return document.documentElement.dataset.theme === 'dark' ? 'dark' : 'light'
}

export default function ThemeToggle() {
  const [theme, setTheme] = useState<Theme>(themeCourant)

  useEffect(() => {
    document.documentElement.dataset.theme = theme
    try {
      localStorage.setItem(CLE, theme)
    } catch {
      // Stockage refusé (navigation privée) : le thème reste valable pour la
      // session, la préférence système reprend la main au rechargement.
    }
  }, [theme])

  const sombre = theme === 'dark'

  return (
    <button
      type="button"
      onClick={() => setTheme(sombre ? 'light' : 'dark')}
      aria-label={sombre ? 'Passer au thème clair' : 'Passer au thème sombre'}
      title={sombre ? 'Thème clair' : 'Thème sombre'}
      className="inline-flex h-7 w-7 shrink-0 items-center justify-center rounded-lg text-ink-soft transition-colors hover:bg-surface-soft hover:text-ink"
    >
      {sombre ? (
        <svg viewBox="0 0 24 24" className="h-4 w-4" aria-hidden>
          <circle cx="12" cy="12" r="4.2" fill="currentColor" />
          {[0, 45, 90, 135, 180, 225, 270, 315].map((angle) => (
            <line
              key={angle}
              x1="12"
              y1="2.6"
              x2="12"
              y2="5.2"
              stroke="currentColor"
              strokeWidth="1.6"
              strokeLinecap="round"
              transform={`rotate(${angle} 12 12)`}
            />
          ))}
        </svg>
      ) : (
        <svg viewBox="0 0 24 24" className="h-4 w-4" aria-hidden>
          <path
            d="M20 14.2A8.4 8.4 0 1 1 9.8 4a6.8 6.8 0 0 0 10.2 10.2Z"
            fill="currentColor"
          />
        </svg>
      )}
    </button>
  )
}
