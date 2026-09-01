// Shared types for the ProofFlow ecosystem.
// The @scavngr/types package is not built; define needed types inline.

export enum ParticipantRole {
  Recycler = 0,
  Collector = 1,
  Manufacturer = 2,
}

export enum WasteType {
  Paper = 0,
  PetPlastic = 1,
  Plastic = 2,
  Metal = 3,
  Glass = 4,
}

export enum WasteStatus {
  Active = 0,
  Transferred = 1,
  Recycled = 2,
  Verified = 3,
}

export interface Participant {
  address: string
  role: ParticipantRole
  name: string
  latitude: number
  longitude: number
  registered_at: number
}

export interface Waste {
  waste_id: bigint
  waste_type: WasteType
  weight: bigint
  current_owner: string
  latitude: string
  longitude: string
  recycled_timestamp: number
  is_active: boolean
  is_confirmed: boolean
  confirmer: string
}

export interface WasteTransfer {
  from: string
  to: string
  timestamp: number
  note?: string
}

export interface Incentive {
  id: number
  rewarder: string
  waste_type: WasteType
  reward_points: number
  total_budget: number
  remaining_budget: number
  active: boolean
  created_at: number
}

export interface ApiResponse<T> {
  success: boolean
  data: T | null
  error: string | null
}

// Frontend-specific extensions and UI types
export interface UiState {
  sidebarOpen: boolean
  theme: 'light' | 'dark' | 'system'
  notifications: NotificationItem[]
}

export interface NotificationItem {
  id: string
  type: 'success' | 'error' | 'warning' | 'info'
  title: string
  description?: string
  timestamp: number
  read: boolean
}

export interface TableColumn<T = Record<string, unknown>> {
  key: keyof T
  label: string
  sortable?: boolean
  render?: (value: unknown, item: T) => React.ReactNode
}

export interface ModalProps {
  isOpen: boolean
  onClose: () => void
  title?: string
  size?: 'sm' | 'md' | 'lg' | 'xl'
}

export interface FormFieldProps {
  name: string
  label: string
  required?: boolean
  disabled?: boolean
  error?: string
  help?: string
}
