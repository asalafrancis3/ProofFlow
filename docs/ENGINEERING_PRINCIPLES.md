# Engineering Principles

ProofFlow's engineering foundation is built on patterns that prioritize correctness, maintainability, and contributor accessibility. These principles are inherited from a proven Stellar/Soroban codebase and applied to ProofFlow's domain.

---

## Contract

### Modular Architecture

The contract is organized into focused modules rather than a single monolithic file:

```
stellar-contract/src/
├── lib.rs           # Core contract logic and entry points
├── types.rs         # Domain structs and enums
├── errors.rs        # Typed error variants
├── events.rs        # Event emission helpers
├── validation.rs    # Input validation functions
└── storage_utils.rs # Storage key construction and TTL helpers
```

**Why it matters:** Each module has a single responsibility. Types can evolve independently of business logic. Errors are testable and enumerable. Events are decoupled from state transitions.

### Typed Domain Model

All domain entities are defined as explicit Rust structs and enums with `#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]`:

```rust
pub struct Job {
    pub id: u64,
    pub client: Address,
    pub title: String,
    pub status: JobStatus,
    pub milestone_count: u32,
    pub total_funded: u128,
    pub created_at: u64,
    pub updated_at: u64,
}
```

**Why it matters:** Type safety prevents entire categories of bugs. Serde serialization ensures deterministic on-chain storage. Enum variants make state machines explicit.

### Explicit Error Variants

Every failure mode has a named error variant with a numeric code:

```rust
pub enum Error {
    NotAuthorized = 1,
    UserAlreadyRegistered = 2,
    JobNotFound = 3,
    JobNotDraft = 4,
    InsufficientFunds = 5,
    // ...
}
```

**Why it matters:** Clients can match on specific error codes. Testing can assert on exact failure modes. No stringly-typed errors leak into the API.

### Deterministic Storage Keys

Storage keys use composite tuples of `(Symbol, ...)`:

```rust
fn user_key(address: &Address) -> (Symbol, Address) {
    (symbol_short!("USR"), address.clone())
}

fn milestone_key(job_id: u64, idx: u32) -> (Symbol, u64, u32) {
    (symbol_short!("MS"), job_id, idx)
}
```

**Why it matters:** Keys are deterministic and human-readable in logs. The Symbol prefix prevents key collisions between entity types. Tuple composition makes key structure explicit.

### Event Architecture

Events use typed symbols (≤9 characters) with structured payloads:

```rust
pub fn emit_job_created(env: &Env, job: &Job) {
    env.events().publish(
        (symbol_short!("JOB_CR"), job.id.clone()),
        (&job.client, &job.title, job.total_funded.clone(), job.created_at),
    );
}
```

**Why it matters:** Events are the indexer's primary data source. Typed symbols enable efficient decoding. Structured payloads avoid serialization ambiguity.

### Authorization Model

Authorization is checked at the entry point, not scattered through business logic:

```rust
pub fn create_job(env: &Env, caller: Address, ...) -> Result<u64, Error> {
    caller.require_auth(); // Explicit auth check at entry
    // ... business logic follows
}
```

**Why it matters:** Auth failures are immediate and clear. Business logic doesn't need to know about authorization. Testing can verify auth behavior independently.

### State Machine Validation

Status transitions are validated explicitly:

```rust
if job.status != JobStatus::Draft {
    return Err(Error::JobNotDraft);
}
```

**Why it matters:** Invalid state transitions are caught at the boundary. The contract never enters an inconsistent state. Testing can verify every valid and invalid transition.

### Accounting Invariants

Escrow operations maintain mathematical invariants:

```rust
// After release: total_funded == total_released + remaining
// After freeze: total_frozen <= total_funded - total_released
```

**Why it matters:** Financial correctness is non-negotiable. Invariants are checked in tests. The contract cannot enter a state where funds are created or destroyed.

---

## Backend

### Domain/Service Separation

The backend separates domain models from service logic:

```
backend/src/
├── services/
│   ├── domain.rs      # Pure data types (no business logic)
│   ├── error_model.rs # Error hierarchy with HTTP mapping
│   └── ...
├── api/
│   ├── proofflow.rs   # Route handlers (thin, delegate to services)
│   └── ...
```

**Why it matters:** Domain types can be shared between API, indexer, and tests. Service logic is testable without HTTP. Route handlers are thin and focused.

### Contract Adapter Abstraction

The adapter provides an async interface to the on-chain contract:

```rust
#[async_trait]
pub trait ProofFlowContractAdapter: Send + Sync {
    async fn create_job(&self, client: &str, ...) -> Result<u64, ContractError>;
    async fn query_job(&self, job_id: u64) -> Result<Job, ContractError>;
    // ...
}
```

**Why it matters:** The adapter decouples the backend from direct Soroban RPC calls. Testing can mock the adapter. Different adapters can target testnet, mainnet, or local networks.

### Structured Errors

Every service error maps to an HTTP status code:

```rust
pub enum ServiceError {
    Contract(ContractError),       // 502 Bad Gateway
    Validation(ValidationError),   // 400 Bad Request
    Auth(AuthError),               // 401/403
    NotFound(NotFoundError),       // 404
    Conflict(ConflictError),       // 409
    State(StateError),             // 409
    Infrastructure(InfraError),    // 500/503
}
```

**Why it matters:** HTTP clients get meaningful status codes. Error responses are consistent. Monitoring can categorize failures by type.

### API Envelope

All responses use a consistent envelope:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

**Why it matters:** Clients can always expect the same structure. Error handling is uniform. No implicit error formats.

---

## Indexer

### Event Decoding

The decoder maps on-chain events to typed Rust structs:

```rust
pub fn decode_event(topics: &[Val], data: &Val) -> Option<DecodedEvent> {
    match topics[0] {
        x if x == symbol_short!("JOB_CR") => Some(DecodedEvent::JobCreated(...)),
        x if x == symbol_short!("MS_SUB") => Some(DecodedEvent::MilestoneSubmitted(...)),
        // ...
    }
}
```

**Why it matters:** Typed decoding prevents runtime panics from malformed events. Missing events are logged and skipped, not crashed. New event types can be added without changing existing handlers.

### Deterministic Processing

Events are processed in order with deterministic state transitions:

```
Event → Decode → Validate → Apply → Persist
```

**Why it matters:** Replaying the same events produces the same state. Debugging is reproducible. No hidden state dependencies.

### Idempotency

Event processing is idempotent — processing the same event twice produces the same result:

```rust
// Check if event already processed
if self.already_processed(&event_id).await? {
    return Ok(());
}
```

**Why it matters:** Network failures can cause event re-delivery. Idempotency prevents duplicate state changes. Recovery is safe.

### Persistence Model

Events project into queryable state:

```
Events (append-only) → Projections (queryable state)
```

**Why it matters:** The event log is the source of truth. Projections can be rebuilt from events. Query performance is independent of event volume.

---

## Frontend

### Reusable Design System

UI components follow a consistent pattern:

```tsx
// Compound component pattern
<Select>
  <SelectTrigger>
    <SelectValue placeholder="..." />
  </SelectTrigger>
  <SelectContent>
    <SelectItem value="...">...</SelectItem>
  </SelectContent>
</Select>
```

**Why it matters:** Components compose naturally. Accessibility is built in. Styling is consistent.

### API Client Abstraction

All API calls go through a typed client:

```typescript
const client = createApiClient({ baseUrl: '/api/v1' })
const result = await client.get<Job[]>('/jobs', { params: { status: 'active' } })
```

**Why it matters:** API calls are type-safe. Base URL is configurable. Error handling is centralized.

### React Query Patterns

Data fetching uses React Query for caching and invalidation:

```typescript
export function useJobs(filters?: JobFilters) {
  return useQuery({
    queryKey: ['jobs', filters],
    queryFn: () => proofflowClient.getJobs(filters),
  })
}
```

**Why it matters:** Caching prevents redundant requests. Invalidation keeps data fresh. Loading/error states are handled uniformly.

### Role-Based Workflows

Navigation and actions are filtered by user role:

```typescript
const NAV_LINKS = [
  { label: 'Dashboard', href: '/dashboard', roles: ['client', 'worker', 'verifier', 'admin'] },
  { label: 'Create Job', href: '/jobs/new', roles: ['client'] },
  // ...
]
```

**Why it matters:** Users only see relevant actions. Authorization is enforced at the UI layer. No dead-end navigation.

### Error/Loading/Empty States

Every data-dependent page handles all three states:

```tsx
if (isLoading) return <PageSkeleton />
if (error) return <ErrorState message={error.message} />
if (items.length === 0) return <EmptyState message="No jobs found" />
return <Content items={items} />
```

**Why it matters:** Users never see a blank screen. Error messages are actionable. Empty states guide next steps.

---

## Testing

### Contract Tests

Every entry point has unit tests covering:

1. Happy path (valid inputs, correct state)
2. Authorization (wrong caller, missing auth)
3. State machine (wrong status for operation)
4. Edge cases (zero amounts, empty strings, max values)
5. Error codes (exact error variant returned)

```rust
#[test]
fn create_job_works() {
    let env = Env::default();
    let contract = env.register_contract(None, ProofFlow);
    let client = ProofFlowClient::new(&env, &contract);
    // ... setup and assertions
}
```

**Why it matters:** Contract behavior is verified before deployment. Regressions are caught immediately. Tests serve as executable documentation.

### Negative Path Testing

For every valid operation, there are tests for every way it can fail:

```rust
#[test]
fn create_job_fails_not_client() {
    // Verify that non-Client roles cannot create jobs
}

#[test]
fn create_job_fails_empty_title() {
    // Verify that empty titles are rejected
}
```

**Why it matters:** Security vulnerabilities are often in error paths. Testing error conditions prevents bypasses.

### Frontend Workflow Tests

Critical user flows are tested end-to-end:

```typescript
it('renders the create form', async () => {
  renderWithProviders(<CreateJobPage />)
  await waitFor(() => {
    expect(screen.getByText('Create Job', { selector: 'h1' })).toBeInTheDocument()
  })
})
```

**Why it matters:** Visual regressions are caught. Component interactions are verified. User workflows are validated.

### API Contract Alignment

Frontend types are verified to match backend types:

| Frontend Type | Backend Type | Match |
|---------------|-------------|-------|
| `UserRole` | `UserRole` | ✅ |
| `JobStatus` | `JobStatus` | ✅ |
| `Job` | `Job` | ✅ |
| ... | ... | ✅ |

**Why it matters:** Serialization mismatches are caught at build time. API changes are immediately visible. No runtime type surprises.

---

## Documentation

### Domain Model

The domain model is documented separately from code:

- `docs/DOMAIN_MODEL.md` — Entities, state machines, invariants
- `docs/ARCHITECTURE.md` — System layers and data flow
- Contract source — Authoritative implementation

**Why it matters:** Documentation provides the "why" behind implementation decisions. New contributors can understand the domain before reading code.

### Provenance

Historical records are preserved:

- `docs/PROVENANCE.md` — What was inherited, what was changed
- `docs/UPSTREAM_BASELINE.md` — Exact upstream commit and fork details
- `LICENSE` — Copyright attribution

**Why it matters:** Legal compliance requires accurate attribution. Contributors understand the codebase history. No misleading claims about independent authorship.

### Contributor Roadmap

Future work is prioritized by value:

- High-value: Contract hardening, persistent indexer, E2E tests
- Medium-value: Analytics, performance, SDK improvements
- Lower-value: i18n, onboarding polish

**Why it matters:** Contributors can find meaningful work. Expectations are clear. No artificial tasks.
