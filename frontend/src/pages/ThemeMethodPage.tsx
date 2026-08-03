import { useQuery } from '@tanstack/react-query'
import { Link } from 'react-router'
import type { MethodResponse } from '../types/themes'

function Row({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="flex items-baseline justify-between border-b border-gray-800 py-1.5">
      <span className="text-sm text-gray-400">{label}</span>
      <span className="text-sm tabular-nums text-gray-200">{value}</span>
    </div>
  )
}

/** CU-06 — Méthode publiée (PROJECT.md §5, §9). */
export default function ThemeMethodPage() {
  const method = useQuery({
    queryKey: ['themes', 'method'],
    queryFn: (): Promise<MethodResponse> =>
      fetch('/api/themes/method').then((res) => {
        if (!res.ok) throw new Error('Chargement impossible')
        return res.json()
      }),
  })

  if (method.isLoading) return <p className="text-gray-500">Chargement…</p>
  if (method.error)
    return <p className="text-red-400">{(method.error as Error).message}</p>

  const data = method.data!

  return (
    <div className="space-y-8">
      <div>
        <Link to="/themes" className="text-sm text-gray-500 hover:text-gray-300">
          ← Tous les thèmes
        </Link>
        <h2 className="mt-2 text-xl font-bold">Méthode de thématisation</h2>
        <p className="mt-2 text-sm text-gray-400">{data.method_note}</p>
      </div>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">
          Ce qui porte le thème
        </h3>
        <p className="text-sm text-gray-400">{data.extraction_rule}</p>
        <p className="mt-2 text-sm text-gray-400">
          Un scrutin reçoit les familles du texte qu'il met aux voix. Le lien
          entre un dossier législatif et un texte est établi par les scrutins
          que la source rattache aux deux — aucun rapprochement de libellés
          n'est deviné.
        </p>
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">
          Ce que fait le modèle
        </h3>
        <p className="text-sm text-gray-400">{data.model_scope}</p>
        <p className="mt-2 text-sm text-gray-400">
          Un texte porte {data.max_families_per_text} familles au plus. La
          famille « Société / libertés » rattache sur l'objet du texte, jamais
          sur son orientation.
        </p>
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">Couverture</h3>
        <Row label="Textes débattus" value={data.texts_total} />
        <Row label="Textes rattachés à au moins une famille" value={data.texts_assigned} />
        <Row label="dont arbitrés par un humain" value={data.texts_arbitrated} />
        <Row
          label="Rattachements automatiques non encore arbitrés"
          value={data.texts_awaiting_arbitration}
        />
        <Row
          label="Textes soumis, aucune famille retenue"
          value={data.texts_without_family}
        />
        <Row
          label="Textes soumis, modèle sans réponse"
          value={data.texts_attempt_failed}
        />
        <Row label="Textes jamais soumis" value={data.texts_never_attempted} />
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">Scrutins</h3>
        <Row label="Scrutins publiés par la source" value={data.scrutins_total} />
        <Row label="Scrutins rattachés à un texte" value={data.scrutins_with_text} />
        <Row
          label="Objets ne nommant aucun texte"
          value={data.scrutins_without_text}
        />
        <Row label="Scrutins portant au moins une famille" value={data.scrutins_assigned} />
        <p className="mt-2 text-xs text-gray-500">
          Les scrutins sans texte et les textes sans famille restent
          consultables :{' '}
          <Link
            to="/themes/non-rattaches"
            className="underline hover:text-gray-300"
          >
            liste des non rattachés
          </Link>
          .
        </p>
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">Dossiers</h3>
        <Row label="Dossiers ingérés" value={data.dossiers_total} />
        <Row label="Dossiers reliés à un texte voté" value={data.dossiers_linked_to_text} />
        <Row label="Dossiers portant une famille" value={data.dossiers_assigned} />
      </section>

      <section>
        <h3 className="mb-2 text-sm font-semibold text-gray-300">
          Par famille
        </h3>
        {data.families.map((family) => (
          <Row
            key={family.code}
            label={family.label}
            value={`${family.text_count} textes · ${family.scrutin_count} scrutins · ${family.arbitrated_text_count} arbitrés`}
          />
        ))}
      </section>
    </div>
  )
}
