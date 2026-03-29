<script lang="ts">
  import { onMount } from 'svelte';

  let { vs = 0 }: { vs: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayValue = $state(0);
  let targetValue = $state(0);
  let animId: number;

  $effect(() => {
    targetValue = vs;
  });

  onMount(() => {
    function animate() {
      displayValue += (targetValue - displayValue) * 0.12;
      draw();
      animId = requestAnimationFrame(animate);
    }
    animate();
    return () => cancelAnimationFrame(animId);
  });

  function draw() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d')!;
    const w = canvas.width;
    const h = canvas.height;
    const cx = w / 2;
    const cy = h / 2;
    const r = Math.min(cx, cy) * 0.92;

    ctx.clearRect(0, 0, w, h);
    ctx.save();
    ctx.translate(cx, cy);

    // Background
    const bg = ctx.createRadialGradient(0, 0, 0, 0, 0, r);
    bg.addColorStop(0, '#1a1a1a');
    bg.addColorStop(1, '#333');
    ctx.beginPath();
    ctx.arc(0, 0, r, 0, Math.PI * 2);
    ctx.fillStyle = bg;
    ctx.fill();

    // Bezel
    ctx.lineWidth = r * 0.08;
    const bezel = ctx.createLinearGradient(-r, -r, r, r);
    bezel.addColorStop(0, '#666');
    bezel.addColorStop(0.5, '#aaa');
    bezel.addColorStop(1, '#666');
    ctx.strokeStyle = bezel;
    ctx.stroke();

    // VSI: 0 at 9 o'clock (180 deg), climb goes UP (counterclockwise to 12), descent DOWN (clockwise to 6)
    // Scale: 0 at 9 o'clock, markings: 5, 10, 15, 20 (x100 fpm)
    // Non-linear scale: more spread near 0, compressed at extremes
    function vsToAngle(fpm: number): number {
      // 0 fpm = 9 o'clock = PI (180 deg)
      // +2000 fpm = ~1 o'clock
      // -2000 fpm = ~5 o'clock
      // Use non-linear mapping for realism
      const maxAngle = Math.PI * 0.7; // total sweep each side
      const normalized = fpm / 2000;
      // Non-linear: more space near zero
      const curved = Math.sign(normalized) * Math.pow(Math.abs(normalized), 0.7);
      return Math.PI - curved * maxAngle;
    }

    // Tick marks and numbers
    const markValues = [0, 500, 1000, 1500, 2000];
    ctx.strokeStyle = '#fff';
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    for (const val of markValues) {
      for (const sign of val === 0 ? [1] : [-1, 1]) {
        const fpm = val * sign;
        const angle = vsToAngle(fpm);
        const cos = Math.cos(angle);
        const sin = Math.sin(angle);
        const isMajor = val % 1000 === 0 || val === 500;
        const innerR = isMajor ? r * 0.6 : r * 0.68;
        const outerR = r * 0.75;

        ctx.beginPath();
        ctx.moveTo(cos * innerR, sin * innerR);
        ctx.lineTo(cos * outerR, sin * outerR);
        ctx.lineWidth = isMajor ? r * 0.025 : r * 0.012;
        ctx.stroke();

        if (val > 0 && val % 500 === 0) {
          const textR = r * 0.5;
          const label = (val / 100).toString();
          ctx.font = `bold ${r * 0.13}px sans-serif`;
          ctx.fillText(label, cos * textR, sin * textR);
        }
      }
    }

    // "UP" and "DOWN" labels
    ctx.font = `bold ${r * 0.08}px sans-serif`;
    ctx.fillStyle = '#fff';
    ctx.fillText('UP', r * 0.15, -r * 0.35);
    ctx.fillText('DN', r * 0.15, r * 0.35);

    // "0" label at 9 o'clock
    ctx.font = `bold ${r * 0.13}px sans-serif`;
    ctx.fillText('0', -r * 0.5, 0);

    // Label
    ctx.font = `bold ${r * 0.08}px sans-serif`;
    ctx.fillText('VERTICAL SPEED', 0, r * 0.15);
    ctx.font = `${r * 0.06}px sans-serif`;
    ctx.fillText('100 FEET PER MIN', 0, r * 0.25);

    // Needle
    const clamped = Math.max(-2000, Math.min(2000, displayValue));
    const needleAngle = vsToAngle(clamped);

    ctx.save();
    ctx.rotate(needleAngle);
    ctx.shadowColor = 'rgba(0,0,0,0.5)';
    ctx.shadowBlur = r * 0.05;
    ctx.shadowOffsetX = r * 0.02;
    ctx.shadowOffsetY = r * 0.02;

    ctx.beginPath();
    ctx.moveTo(r * 0.68, 0);
    ctx.lineTo(-r * 0.15, -r * 0.02);
    ctx.lineTo(-r * 0.15, r * 0.02);
    ctx.closePath();
    ctx.fillStyle = '#fff';
    ctx.fill();
    ctx.strokeStyle = '#111';
    ctx.lineWidth = r * 0.006;
    ctx.stroke();
    ctx.restore();

    // Center cap
    const capGrad = ctx.createRadialGradient(0, 0, 0, 0, 0, r * 0.06);
    capGrad.addColorStop(0, '#ddd');
    capGrad.addColorStop(1, '#555');
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.06, 0, Math.PI * 2);
    ctx.fillStyle = capGrad;
    ctx.fill();

    // Glass effect
    const glass = ctx.createRadialGradient(0, -r * 0.3, 0, 0, 0, r);
    glass.addColorStop(0, 'rgba(255,255,255,0.04)');
    glass.addColorStop(1, 'rgba(0,0,0,0.15)');
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.95, 0, Math.PI * 2);
    ctx.fillStyle = glass;
    ctx.fill();

    ctx.restore();
  }
</script>

<canvas bind:this={canvas} width="250" height="250" style="width:100%;height:100%;"></canvas>
