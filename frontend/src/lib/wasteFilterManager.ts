import { useState, useCallback, useMemo } from 'react';

export interface WasteFilter {
  wasteType?: string[];
  status?: string[];
  dateRange?: {
    start: Date;
    end: Date;
  };
  weight?: {
    min: number;
    max: number;
  };
  location?: {
    lat: number;
    lon: number;
    radius: number;
  };
  verificationStatus?: 'verified' | 'unverified' | 'all';
}

export interface FilterPreset {
  id: string;
  name: string;
  filters: WasteFilter;
}

interface WasteFilterManagerProps {
  onFilterChange: (filters: WasteFilter) => void;
  presets?: FilterPreset[];
  onSavePreset?: (preset: FilterPreset) => void;
}

const DEFAULT_PRESETS: FilterPreset[] = [
  {
    id: 'recent',
    name: 'Recent Items',
    filters: {
      dateRange: {
        start: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
        end: new Date(),
      },
    },
  },
  {
    id: 'high-weight',
    name: 'High Weight',
    filters: {
      weight: { min: 100, max: Infinity },
    },
  },
  {
    id: 'verified',
    name: 'Verified Only',
    filters: {
      verificationStatus: 'verified',
    },
  },
];

export function useWasteFilterManager({
  onFilterChange,
  presets = DEFAULT_PRESETS,
  onSavePreset,
}: WasteFilterManagerProps) {
  const [filters, setFilters] = useState<WasteFilter>({});
  const [savedPresets, setSavedPresets] = useState<FilterPreset[]>(presets);

  const updateFilter = useCallback(
    (newFilters: Partial<WasteFilter>) => {
      const updated = { ...filters, ...newFilters };
      setFilters(updated);
      onFilterChange(updated);
    },
    [filters, onFilterChange]
  );

  const clearFilters = useCallback(() => {
    setFilters({});
    onFilterChange({});
  }, [onFilterChange]);

  const applyPreset = useCallback(
    (presetId: string) => {
      const preset = savedPresets.find((p) => p.id === presetId);
      if (preset) {
        setFilters(preset.filters);
        onFilterChange(preset.filters);
      }
    },
    [savedPresets, onFilterChange]
  );

  const saveCurrentAsPreset = useCallback(
    (name: string) => {
      const newPreset: FilterPreset = {
        id: `preset-${Date.now()}`,
        name,
        filters,
      };
      setSavedPresets([...savedPresets, newPreset]);
      onSavePreset?.(newPreset);
    },
    [filters, savedPresets, onSavePreset]
  );

  const isFilterActive = useMemo(() => {
    return Object.keys(filters).length > 0;
  }, [filters]);

  return {
    filters,
    updateFilter,
    clearFilters,
    applyPreset,
    saveCurrentAsPreset,
    isFilterActive,
    presets: savedPresets,
  };
};

export default useWasteFilterManager;
