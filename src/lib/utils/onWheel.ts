export function onWheel(e: WheelEvent) {
  const input = e.currentTarget as HTMLInputElement;
  if (document.activeElement !== input) return;
  e.preventDefault();
  const step = parseFloat(input.step) || 1;
  const delta = e.deltaY < 0 ? step : -step;
  const min = parseFloat(input.min);
  const max = parseFloat(input.max);
  let val = parseFloat(input.value) + delta;
  if (!isNaN(min)) val = Math.max(min, val);
  if (!isNaN(max)) val = Math.min(max, val);
  input.value = String(Math.round(val * 100) / 100);
  input.dispatchEvent(new Event('input', { bubbles: true }));
}
