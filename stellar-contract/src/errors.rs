use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    // ── General ──────────────────────────────────────────────────────────────
    NotAuthorized = 1,
    NotFound = 2,
    InvalidInput = 3,
    AlreadyInitialized = 4,
    ContractPaused = 5,

    // ── Job errors ───────────────────────────────────────────────────────────
    JobNotFound = 10,
    JobNotDraft = 11,
    JobNotFunded = 12,
    JobNotActive = 13,
    JobAlreadySettled = 14,
    JobAlreadyCancelled = 15,
    JobCannotBeCancelled = 16,
    JobNotEnoughMilestones = 17,

    // ── Escrow errors ────────────────────────────────────────────────────────
    EscrowNotFound = 20,
    InsufficientFunds = 21,
    EscrowAlreadyFunded = 22,
    EscrowFrozen = 23,
    EscrowOverRelease = 24,
    EscrowNotFullyFunded = 25,

    // ── Milestone errors ─────────────────────────────────────────────────────
    MilestoneNotFound = 30,
    MilestoneAlreadySubmitted = 31,
    MilestoneNotSubmitted = 32,
    MilestoneNotApproved = 33,
    MilestoneAlreadyReleased = 34,
    MilestoneAlreadyDisputed = 35,
    MilestoneCannotSubmit = 36,
    InvalidMilestoneIndex = 37,

    // ── Submission errors ────────────────────────────────────────────────────
    SubmissionAlreadyExists = 40,
    EvidenceRequired = 41,

    // ── Attestation errors ───────────────────────────────────────────────────
    NotWhitelistedVerifier = 50,
    AttestationAlreadyExists = 51,
    CannotAttestOwnWork = 52,

    // ── Dispute errors ───────────────────────────────────────────────────────
    DisputeNotFound = 60,
    DisputeAlreadyResolved = 61,
    MilestoneNotDisputed = 62,
    CannotDisputeReleasedMilestone = 63,

    // ── Reputation errors ────────────────────────────────────────────────────
    ReputationNotFound = 70,

    // ── Admin errors ─────────────────────────────────────────────────────────
    VerifierAlreadyWhitelisted = 80,
    VerifierNotWhitelisted = 81,
    InvalidConfigKey = 82,

    // ── Replay protection ────────────────────────────────────────────────────
    NonceAlreadyUsed = 90,
}
