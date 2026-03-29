// Import all bundled checklists eagerly
const modules = import.meta.glob('../../checklists/*.json', { eager: true });

export interface AutoDetect {
  var_name: string;
  var_type: string;
  condition: string;
  xplane_dataref?: string;
}

export interface ChecklistItem {
  id: string;
  label: string;
  expected: string;
  how_to?: string;
  location?: string;
  auto_detect?: AutoDetect | null;
}

export interface ChecklistPhase {
  id: string;
  name: string;
  items: ChecklistItem[];
}

export interface Checklist {
  id: string;
  name: string;
  author: string;
  version: string;
  category: string;
  simulator?: string;
  phases: ChecklistPhase[];
}

let cachedChecklists: Checklist[] | null = null;

export function getChecklists(): Checklist[] {
  if (cachedChecklists) return cachedChecklists;
  cachedChecklists = Object.values(modules).map((m: any) => m.default ?? m) as Checklist[];
  cachedChecklists.sort((a, b) => a.name.localeCompare(b.name));
  return cachedChecklists;
}

export function getChecklist(id: string): Checklist | undefined {
  return getChecklists().find((c) => c.id === id);
}

// Persistence helpers
const STORAGE_KEY = 'sf-checked';

export interface CheckedState {
  [checklistId: string]: {
    [phaseId: string]: string[];
  };
}

export function loadCheckedState(): CheckedState {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
}

export function saveCheckedState(state: CheckedState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {}
}
