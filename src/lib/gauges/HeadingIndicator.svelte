<script lang="ts">
  import { onMount } from 'svelte';

  let { heading = 0 }: { heading: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayValue = $state(0);
  let targetValue = $state(0);
  let animId: number;

  $effect(() => {
    targetValue = heading;
  });

  onMount(() => {
    function animate() {
      // Handle wrap-around for heading (shortest path)
      let diff = targetValue - displayValue;
      if (diff > 180) diff -= 360;
      if (diff < -180) diff += 360;
      displayValue += diff * 0.15;
      if (displayValue < 0) displayValue += 360;
      if (displayValue >= 360) displayValue -= 360;
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

    // Rotating compass card
    ctx.save();
    ctx.rotate((-displayValue * Math.PI) / 180);

    const cardR = r * 0.78;
    const cardLabels: Record<number, string> = {
      0: 'N', 90: 'E', 180: 'S', 270: 'W'
    };

    for (let deg = 0; deg < 360; deg += 5) {
      const angle = (deg * Math.PI) / 180 - Math.PI / 2;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      const isMajor = deg % 30 === 0;
      const isMedium = deg % 10 === 0;
      const innerR = isMajor ? cardR * 0.78 : isMedium ? cardR * 0.85 : cardR * 0.9;

      ctx.beginPath();
      ctx.moveTo(cos * innerR, sin * innerR);
      ctx.lineTo(cos * cardR, sin * cardR);
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = isMajor ? r * 0.02 : r * 0.008;
      ctx.stroke();

      if (isMajor) {
        const textR = cardR * 0.65;
        ctx.save();
        ctx.translate(cos * textR, sin * textR);
        ctx.rotate(angle + Math.PI / 2);
        ctx.fillStyle = '#fff';

        if (cardLabels[deg]) {
          ctx.font = `bold ${r * 0.16}px sans-serif`;
          ctx.fillStyle = deg === 0 ? '#ff8800' : '#fff';
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText(cardLabels[deg], 0, 0);
        } else {
          ctx.font = `bold ${r * 0.11}px sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';
          ctx.fillText((deg / 10).toString(), 0, 0);
        }
        ctx.restore();
      }
    }
    ctx.restore();

    // Fixed lubber line at top
    ctx.beginPath();
    ctx.moveTo(0, -r * 0.85);
    ctx.lineTo(-r * 0.04, -r * 0.92);
    ctx.lineTo(r * 0.04, -r * 0.92);
    ctx.closePath();
    ctx.fillStyle = '#ff8800';
    ctx.fill();

    // Fixed aircraft symbol in center
    ctx.strokeStyle = '#ff8800';
    ctx.lineWidth = r * 0.025;
    ctx.lineCap = 'round';
    // Fuselage
    ctx.beginPath();
    ctx.moveTo(0, r * 0.15);
    ctx.lineTo(0, -r * 0.2);
    ctx.stroke();
    // Wings
    ctx.beginPath();
    ctx.moveTo(-r * 0.2, r * 0.02);
    ctx.lineTo(r * 0.2, r * 0.02);
    ctx.stroke();
    // Tail
    ctx.beginPath();
    ctx.moveTo(-r * 0.08, r * 0.15);
    ctx.lineTo(r * 0.08, r * 0.15);
    ctx.stroke();

    // Label
    ctx.fillStyle = '#fff';
    ctx.font = `bold ${r * 0.07}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.fillText('GYRO', 0, r * 0.4);

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
