export function gradeFromRate(r: number) {
  const a = Math.abs(r);
  return a < 60 ? 'A+' : a < 120 ? 'A' : a < 200 ? 'B' : a < 300 ? 'C' : a < 500 ? 'D' : 'F';
}

export function gradeColor(g: string) {
  return g === 'A+' || g === 'A' ? 'var(--color-green)' : g === 'B' ? 'var(--color-accent)' : g === 'C' ? 'var(--color-yellow)' : 'var(--color-red)';
}

export function landingVerdict(r: number): string {
  const a = Math.abs(r);
  return a < 30 ? 'GREASER' : a < 60 ? 'BUTTER' : a < 120 ? 'SMOOTH' : a < 200 ? 'NORMAL' : a < 300 ? 'FIRM' : a < 500 ? 'HARD' : 'CRASHED';
}

export function landingGrade(rate: number): { grade: string; color: string } {
  const abs = Math.abs(rate);
  if (abs <= 60) return { grade: 'BUTTER', color: 'var(--color-green)' };
  if (abs <= 120) return { grade: 'SMOOTH', color: 'var(--color-green)' };
  if (abs <= 200) return { grade: 'GOOD', color: 'var(--color-accent)' };
  if (abs <= 300) return { grade: 'FIRM', color: 'var(--color-yellow)' };
  if (abs <= 500) return { grade: 'HARD', color: 'var(--color-red)' };
  return { grade: 'CRASH', color: 'var(--color-red)' };
}

export function arcPath(cx: number, cy: number, r: number, startAngle: number, endAngle: number): string {
  const s = startAngle * Math.PI / 180, e = endAngle * Math.PI / 180;
  const x1 = cx + r * Math.cos(s), y1 = cy + r * Math.sin(s);
  const x2 = cx + r * Math.cos(e), y2 = cy + r * Math.sin(e);
  const large = endAngle - startAngle > 180 ? 1 : 0;
  return `M ${x1} ${y1} A ${r} ${r} 0 ${large} 1 ${x2} ${y2}`;
}

export function needlePos(rate: number): number {
  return Math.min(180, Math.max(0, Math.abs(rate) / 500 * 180));
}
