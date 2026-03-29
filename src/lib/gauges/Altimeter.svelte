<script lang="ts">
  import { onMount } from 'svelte';

  let { altitude = 0, baro = 29.92 }: { altitude: number; baro: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayValue = $state(0);
  let targetValue = $state(0);
  let animId: number;

  $effect(() => {
    targetValue = altitude;
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

    // Dial markings: 0-9 around full circle (each = 100 ft)
    ctx.strokeStyle = '#fff';
    ctx.fillStyle = '#fff';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    for (let i = 0; i < 50; i++) {
      const angle = (i / 50) * Math.PI * 2 - Math.PI / 2;
      const isMajor = i % 5 === 0;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      const innerR = isMajor ? r * 0.62 : r * 0.7;
      const outerR = r * 0.76;

      ctx.beginPath();
      ctx.moveTo(cos * innerR, sin * innerR);
      ctx.lineTo(cos * outerR, sin * outerR);
      ctx.lineWidth = isMajor ? r * 0.02 : r * 0.008;
      ctx.stroke();

      if (isMajor) {
        const digit = i / 5;
        const textR = r * 0.53;
        ctx.font = `bold ${r * 0.14}px sans-serif`;
        ctx.fillText(digit.toString(), cos * textR, sin * textR);
      }
    }

    // Label
    ctx.font = `bold ${r * 0.09}px sans-serif`;
    ctx.fillStyle = '#fff';
    ctx.fillText('ALTITUDE', 0, -r * 0.2);
    ctx.font = `${r * 0.065}px sans-serif`;
    ctx.fillText('100 FEET', 0, -r * 0.1);

    // Kohlsman window (baro setting)
    ctx.save();
    const kwX = r * 0.35;
    const kwY = r * 0.05;
    const kwW = r * 0.3;
    const kwH = r * 0.14;
    ctx.fillStyle = '#111';
    ctx.strokeStyle = '#888';
    ctx.lineWidth = r * 0.01;
    ctx.fillRect(kwX - kwW / 2, kwY - kwH / 2, kwW, kwH);
    ctx.strokeRect(kwX - kwW / 2, kwY - kwH / 2, kwW, kwH);
    ctx.fillStyle = '#fff';
    ctx.font = `bold ${r * 0.09}px sans-serif`;
    ctx.fillText(baro.toFixed(2), kwX, kwY + r * 0.01);
    ctx.restore();

    const alt = Math.max(0, Math.min(20000, displayValue));

    // 10,000s needle (tiny triangle)
    const angle10k = ((alt / 100000) * Math.PI * 2) - Math.PI / 2;
    ctx.save();
    ctx.rotate(angle10k + Math.PI / 2);
    ctx.beginPath();
    ctx.moveTo(0, -r * 0.3);
    ctx.lineTo(-r * 0.04, -r * 0.2);
    ctx.lineTo(r * 0.04, -r * 0.2);
    ctx.closePath();
    ctx.fillStyle = '#fff';
    ctx.fill();
    ctx.strokeStyle = '#111';
    ctx.lineWidth = r * 0.005;
    ctx.stroke();
    ctx.restore();

    // 1,000s needle (short, wide)
    const angle1k = ((alt / 10000) * Math.PI * 2) - Math.PI / 2;
    ctx.save();
    ctx.rotate(angle1k + Math.PI / 2);
    ctx.shadowColor = 'rgba(0,0,0,0.4)';
    ctx.shadowBlur = r * 0.04;
    ctx.shadowOffsetX = r * 0.015;
    ctx.shadowOffsetY = r * 0.015;
    ctx.beginPath();
    ctx.moveTo(0, -r * 0.48);
    ctx.lineTo(-r * 0.04, -r * 0.38);
    ctx.lineTo(-r * 0.025, r * 0.12);
    ctx.lineTo(r * 0.025, r * 0.12);
    ctx.lineTo(r * 0.04, -r * 0.38);
    ctx.closePath();
    ctx.fillStyle = '#fff';
    ctx.fill();
    ctx.strokeStyle = '#111';
    ctx.lineWidth = r * 0.006;
    ctx.stroke();
    ctx.restore();

    // 100s needle (long, thin)
    const angle100 = ((alt / 1000) * Math.PI * 2) - Math.PI / 2;
    ctx.save();
    ctx.rotate(angle100 + Math.PI / 2);
    ctx.shadowColor = 'rgba(0,0,0,0.5)';
    ctx.shadowBlur = r * 0.05;
    ctx.shadowOffsetX = r * 0.02;
    ctx.shadowOffsetY = r * 0.02;
    ctx.beginPath();
    ctx.moveTo(0, -r * 0.7);
    ctx.lineTo(-r * 0.02, r * 0.15);
    ctx.lineTo(r * 0.02, r * 0.15);
    ctx.closePath();
    ctx.fillStyle = '#fff';
    ctx.fill();
    ctx.strokeStyle = '#111';
    ctx.lineWidth = r * 0.005;
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
