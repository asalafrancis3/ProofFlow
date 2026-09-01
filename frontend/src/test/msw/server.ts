import { setupServer } from 'msw/node'
import { proofflowHandlers } from './proofflowHandlers'
import { handlers } from './handlers'

export const server = setupServer(...proofflowHandlers, ...handlers)
