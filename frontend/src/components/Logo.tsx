type LogoProps = {
  className?: string
}

/*
 * L'hémicycle tricolore, sur sa pastille crème.
 *
 * Le triptyque nu (`logo/logo-i-triptyque.svg`) a ses rayons centraux en crème
 * clair : il est fait pour un fond sombre et disparaît sur `bg-surface`. C'est
 * donc la déclinaison sur pastille qui sert dans l'interface, elle porte son
 * propre fond et tient sur n'importe quelle couleur.
 *
 * Inliné plutôt que servi en fichier : neuf tracés pèsent moins qu'une requête,
 * et l'en-tête ne s'affiche pas un instant sans son logo.
 */
export default function Logo({ className }: LogoProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      role="img"
      aria-label="hémicycle.data"
      className={className}
    >
      <rect width="64" height="64" rx="12" fill="#ded3bd" />
      <g transform="translate(3 15.2) scale(0.29)">
        <path
          d="M 36.2 105.0 A 64 64 0 0 1 42.2 82.4 L 72.9 97.1 A 30 30 0 0 0 70.1 107.6 Z"
          fill="#16233f"
        />
        <path
          d="M 27.5 60.2 A 88 88 0 0 1 50.2 37.5 L 83.0 85.3 A 30 30 0 0 0 75.3 93.0 Z"
          fill="#16233f"
        />
        <path
          d="M 56.9 19.7 A 100 100 0 0 1 92.2 10.3 L 97.6 80.1 A 30 30 0 0 0 87.1 82.9 Z"
          fill="#ffffff"
        />
        <path
          d="M 107.8 10.3 A 100 100 0 0 1 143.1 19.7 L 112.9 82.9 A 30 30 0 0 0 102.4 80.1 Z"
          fill="#ffffff"
        />
        <path
          d="M 149.8 37.5 A 88 88 0 0 1 172.5 60.2 L 124.7 93.0 A 30 30 0 0 0 117.0 85.3 Z"
          fill="#9e3038"
        />
        <path
          d="M 157.8 82.4 A 64 64 0 0 1 163.8 105.0 L 129.9 107.6 A 30 30 0 0 0 127.1 97.1 Z"
          fill="#9e3038"
        />
      </g>
    </svg>
  )
}
