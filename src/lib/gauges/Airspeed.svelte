<script lang="ts">
  import { onMount } from 'svelte';

  let { speed = 0 }: { speed: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayValue = $state(0);
  let targetValue = $state(0);
  let animId: number;

  $effect(() => {
    targetValue = speed;
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

    // Angle mapping: 0 kts at ~7 o'clock (210 deg), 200 kts at ~5 o'clock (150+360=510 -> wrap)
    // Full sweep ~300 degrees
    const startAngle = (Math.PI * 7) / 6; // 210 degrees
    const sweepAngle = (Math.PI * 10) / 6; // 300 degrees
    function ktsToAngle(kts: number): number {
      return startAngle + (kts / 200) * sweepAngle;
    }

    // Draw colored arcs
    const arcR = r * 0.78;
    const arcW = r * 0.06;

    function drawArc(fromKts: number, toKts: number, color: string, width: number) {
      ctx.beginPath();
      ctx.arc(0, 0, arcR, ktsToAngle(fromKts) - Math.PI / 2, ktsToAngle(toKts) - Math.PI / 2);
      ctx.strokeStyle = color;
      ctx.lineWidth = width;
      ctx.stroke();
    }

    // White arc (flap range)
    drawArc(40, 85, '#ffffff', arcW);
    // Green arc (normal)
    drawArc(48, 129, '#00cc44', arcW);
    // Yellow arc (caution)
    drawArc(129, 163, '#ffcc00', arcW);
    // Red line (Vne)
    drawArc(163, 165, '#ff3333', arcW * 1.5);

    // Tick marks and numbers
    ctx.strokeStyle = '#fff';
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.font = `bold ${r * 0.12}px sans-serif`;

    for (let kts = 0; kts <= 200; kts += 10) {
      const angle = ktsToAngle(kts) - Math.PI / 2;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      const isMajor = kts % 20 === 0;
      const innerR = isMajor ? r * 0.6 : r * 0.67;
      const outerR = r * 0.72;

      ctx.beginPath();
      ctx.moveTo(cos * innerR, sin * innerR);
      ctx.lineTo(cos * outerR, sin * outerR);
      ctx.lineWidth = isMajor ? r * 0.02 : r * 0.01;
      ctx.strokeStyle = '#fff';
      ctx.stroke();

      if (isMajor && kts > 0) {
        const textR = r * 0.5;
        ctx.fillText(kts.toString(), cos * textR, sin * textR);
      }
    }

    // Label
    ctx.font = `bold ${r * 0.1}px sans-serif`;
    ctx.fillStyle = '#fff';
    ctx.fillText('AIRSPEED', 0, -r * 0.2);
    ctx.font = `${r * 0.07}px sans-serif`;
    ctx.fillText('KNOTS', 0, r * 0.25);

    // Needle shadow
    const needleAngle = ktsToAngle(Math.max(0, Math.min(200, displayValue))) - Math.PI / 2;
    ctx.save();
    ctx.rotate(needleAngle);
    ctx.shadowColor = 'rgba(0,0,0,0.5)';
    ctx.shadowBlur = r * 0.05;
    ctx.shadowOffsetX = r * 0.02;
    ctx.shadowOffsetY = r * 0.02;

    // Needle
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
    glass.addColorStop(0.5, 'rgba(255,255,255,0.01)');
    glass.addColorStop(1, 'rgba(0,0,0,0.15)');
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.95, 0, Math.PI * 2);
    ctx.fillStyle = glass;
    ctx.fill();

    ctx.restore();
  }
</script>

<canvas bind:this={canvas} width="250" height="250" style="width:100%;height:100%;"></canvas>
