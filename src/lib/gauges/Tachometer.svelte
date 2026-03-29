<script lang="ts">
  import { onMount } from 'svelte';

  let { rpm = 0 }: { rpm: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayValue = $state(0);
  let targetValue = $state(0);
  let animId: number;

  $effect(() => {
    targetValue = rpm;
  });

  onMount(() => {
    function animate() {
      displayValue += (targetValue - displayValue) * 0.15;
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

    // Angle mapping: 0 RPM at ~7 o'clock, 3500 RPM at ~5 o'clock
    const startAngle = (Math.PI * 7) / 6;
    const sweepAngle = (Math.PI * 10) / 6;
    function rpmToAngle(v: number): number {
      return startAngle + (v / 3500) * sweepAngle;
    }

    // Colored arcs
    const arcR = r * 0.78;
    const arcW = r * 0.06;

    function drawArc(from: number, to: number, color: string, width: number) {
      ctx.beginPath();
      ctx.arc(0, 0, arcR, rpmToAngle(from) - Math.PI / 2, rpmToAngle(to) - Math.PI / 2);
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.stroke();
    }

    // Green arc (normal operating)
    drawArc(2100, 2700, '#00cc44', arcW);
    // Red line
    drawArc(2700, 2720, '#ff3333', arcW * 1.5);

    // Tick marks and numbers
    ctx.strokeStyle = '#fff';
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    for (let v = 0; v <= 3500; v += 100) {
      const angle = rpmToAngle(v) - Math.PI / 2;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      const isMajor = v % 500 === 0;
      const innerR = isMajor ? r * 0.6 : r * 0.67;
      const outerR = r * 0.72;

      ctx.beginPath();
      ctx.moveTo(cos * innerR, sin * innerR);
      ctx.lineTo(cos * outerR, sin * outerR);
      ctx.lineWidth = isMajor ? r * 0.02 : r * 0.008;
      ctx.strokeStyle = '#fff';
      ctx.stroke();

      if (isMajor) {
        const textR = r * 0.48;
        ctx.font = `bold ${r * 0.11}px sans-serif`;
        ctx.fillText((v / 100).toString(), cos * textR, sin * textR);
      }
    }

    // Labels
    ctx.font = `bold ${r * 0.1}px sans-serif`;
    ctx.fillStyle = '#fff';
    ctx.fillText('RPM', 0, -r * 0.15);
    ctx.font = `${r * 0.07}px sans-serif`;
    ctx.fillText('x100', 0, r * 0.25);

    // Needle
    const needleAngle = rpmToAngle(Math.max(0, Math.min(3500, displayValue))) - Math.PI / 2;
    ctx.save();
    ctx.rotate(needleAngle);
    ctx.shadowColor = 'rgba(0,0,0,0.5)';
    ctx.shadowBlur = r * 0.05;
    ctx.shadowOffsetX = r * 0.02;
    ctx.shadowOffsetY = r * 0.02;

    ctx.beginPath();
    ctx.moveTo(0, -r * 0.68);
    ctx.lineTo(-r * 0.025, r * 0.15);
    ctx.lineTo(r * 0.025, r * 0.15);
    ctx.closePath();
    ctx.fillStyle = '#fff';
    ctx.fill();
    ctx.strokeStyle = '#111';
    ctx.lineWidth = r * 0.008;
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
