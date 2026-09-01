import { Link } from 'react-router-dom'
import { Shield, FileCheck, DollarSign, CheckCircle2, ArrowRight } from 'lucide-react'
import { Button } from '@/components/ui/Button'
import { ThemeToggle } from '@/components/ui/ThemeToggle'
import { useAppTitle } from '@/hooks/useAppTitle'

const STEPS = [
  {
    icon: FileCheck,
    title: 'Create Agreement',
    description: 'Define work, milestones, and payment terms on-chain.',
  },
  {
    icon: DollarSign,
    title: 'Fund Escrow',
    description: 'Lock funds in a smart contract escrow until milestones are met.',
  },
  {
    icon: Shield,
    title: 'Verify & Approve',
    description: 'Independent verifiers review evidence and approve milestones.',
  },
  {
    icon: CheckCircle2,
    title: 'Release & Settle',
    description: 'Milestone payments release automatically upon approval.',
  },
]

const FEATURES = [
  {
    title: 'On-Chain Escrow',
    description: 'Funds are locked in a Soroban smart contract. No trust required — code enforces the agreement.',
  },
  {
    title: 'Independent Verification',
    description: 'Third-party verifiers review evidence and attest to milestone completion.',
  },
  {
    title: 'Reputation System',
    description: 'Build a transparent, on-chain reputation from completed work and verifications.',
  },
  {
    title: 'Dispute Resolution',
    description: 'Built-in arbitration for when agreements go wrong. Fair, transparent, on-chain.',
  },
]

const FOOTER_LINKS = [
  { label: 'GitHub', href: 'https://github.com' },
  { label: 'Docs', href: '/docs' },
]

export function LandingPage() {
  useAppTitle('ProofFlow — Decentralized Verification Protocol')

  return (
    <div className="flex min-h-screen flex-col bg-background text-foreground">
      {/* Nav */}
      <header className="flex h-14 items-center justify-between border-b px-6">
        <div className="flex items-center gap-2 font-bold">
          <Shield className="h-5 w-5 text-primary" aria-hidden="true" />
          ProofFlow
        </div>
        <div className="flex items-center gap-2">
          <ThemeToggle />
          <Button asChild size="sm">
            <Link to="/dashboard">Launch App</Link>
          </Button>
        </div>
      </header>

      {/* Hero */}
      <section className="flex flex-1 flex-col items-center justify-center px-6 py-24 text-center">
        <h1 className="text-4xl font-bold tracking-tight sm:text-5xl lg:text-6xl">
          Work Agreements,<br />
          <span className="text-primary">Verified On-Chain</span>
        </h1>
        <p className="mt-6 max-w-2xl text-lg text-muted-foreground">
          ProofFlow is a decentralized verification and milestone settlement protocol.
          Fund escrow, submit proof, get verified, get paid.
        </p>
        <div className="mt-8 flex gap-4">
          <Button asChild size="lg">
            <Link to="/dashboard">
              Get Started
              <ArrowRight className="ml-2 h-4 w-4" aria-hidden="true" />
            </Link>
          </Button>
          <Button asChild variant="outline" size="lg">
            <Link to="/how-it-works">How It Works</Link>
          </Button>
        </div>
      </section>

      {/* How it works */}
      <section className="border-t bg-muted/40 px-6 py-20">
        <div className="mx-auto max-w-5xl">
          <h2 className="text-center text-3xl font-bold">How ProofFlow Works</h2>
          <div className="mt-12 grid gap-8 sm:grid-cols-2 lg:grid-cols-4">
            {STEPS.map((step, i) => (
              <div key={step.title} className="flex flex-col items-center text-center">
                <div className="flex h-12 w-12 items-center justify-center rounded-full bg-primary/10 text-primary">
                  <step.icon className="h-6 w-6" aria-hidden="true" />
                </div>
                <span className="mt-3 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                  Step {i + 1}
                </span>
                <h3 className="mt-2 text-lg font-semibold">{step.title}</h3>
                <p className="mt-1 text-sm text-muted-foreground">{step.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* Features */}
      <section className="px-6 py-20">
        <div className="mx-auto max-w-5xl">
          <h2 className="text-center text-3xl font-bold">Built for Trust</h2>
          <div className="mt-12 grid gap-8 sm:grid-cols-2">
            {FEATURES.map((f) => (
              <div key={f.title} className="rounded-lg border p-6">
                <h3 className="text-lg font-semibold">{f.title}</h3>
                <p className="mt-2 text-sm text-muted-foreground">{f.description}</p>
              </div>
            ))}
          </div>
        </div>
      </section>

      {/* CTA */}
      <section className="border-t bg-muted/40 px-6 py-20 text-center">
        <h2 className="text-3xl font-bold">Ready to Build?</h2>
        <p className="mt-4 text-muted-foreground">
          Connect your Stellar wallet and start creating verified work agreements.
        </p>
        <Button asChild size="lg" className="mt-8">
          <Link to="/dashboard">
            Launch ProofFlow
            <ArrowRight className="ml-2 h-4 w-4" aria-hidden="true" />
          </Link>
        </Button>
      </section>

      {/* Footer */}
      <footer className="border-t px-6 py-8">
        <div className="mx-auto flex max-w-5xl flex-col items-center justify-between gap-4 sm:flex-row">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Shield className="h-4 w-4" aria-hidden="true" />
            ProofFlow Protocol
          </div>
          <div className="flex gap-4">
            {FOOTER_LINKS.map((link) => (
              <a
                key={link.label}
                href={link.href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-muted-foreground hover:text-foreground"
              >
                {link.label}
              </a>
            ))}
          </div>
        </div>
      </footer>
    </div>
  )
}
