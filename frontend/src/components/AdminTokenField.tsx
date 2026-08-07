import { useState } from 'react'
import { getAdminToken, setAdminToken } from '../lib/adminToken'

/**
 * Champ de saisie du jeton d'administration, partagé par les écrans qui
 * écrivent (sélection, fiche dossier, arbitrage).
 *
 * Le jeton n'est pas un mot de passe de compte : c'est la valeur du jour,
 * dérivée du secret du serveur. Il expire à minuit UTC, d'où le rappel affiché
 * — un opérateur qui prend un 401 en pleine session doit comprendre en une
 * seconde qu'il n'a pas perdu ses droits, seulement la journée.
 */
export function AdminTokenField({ className = '' }: { className?: string }) {
  const [token, setToken] = useState(getAdminToken)

  return (
    <label className={`block text-sm ${className}`}>
      <span className="text-xs text-ink-faint">
        Jeton du jour — <code>deploy/bin/admin-token.sh</code> sur le VPS
      </span>
      <input
        type="password"
        value={token}
        autoComplete="off"
        placeholder="32 caractères hexadécimaux"
        onChange={(e) => {
          setToken(e.target.value)
          setAdminToken(e.target.value)
        }}
        className="mt-0.5 w-full rounded border border-line bg-canvas px-2 py-1 text-sm focus:border-accent focus:outline-none"
      />
    </label>
  )
}
