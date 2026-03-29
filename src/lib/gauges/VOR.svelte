<script lang="ts">
  import { onMount } from 'svelte';

  let { obs = 0, cdi = 0, toFrom = 'OFF', gs = 0 }: { obs: number; cdi: number; toFrom: string; gs: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayCdi = $state(0);
  let displayGs = $state(0);
  let targetCdi = $state(0);
  let targetGs = $state(0);
  let displayObs = $state(0);
  let targetObs = $state(0);
  let animId: number;

  $effect(() => {
    targetCdi = cdi;
    targetGs = gs;
    targetObs = obs;
  });

  onMount(() => {
    function animate() {
      displayCdi += (targetCdi - displayCdi) * 0.15;
      displayGs += (targetGs - displayGs) * 0.15;
      // Handle wrap for OBS
      let diff = targetObs - displayObs;
      if (diff > 180) diff -= 360;
      if (diff < -180) diff += 360;
      displayObs += diff * 0.15;
      if (displayObs < 0) displayObs += 360;
      if (displayObs >= 360) displayObs -= 360;
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

    // Rotating compass card (OBS)
    ctx.save();
    ctx.rotate((-displayObs * Math.PI) / 180);
    const cardR = r * 0.82;
    const cardLabels: Record<number, string> = { 0: 'N', 90: 'E', 180: 'S', 270: 'W' };

    for (let deg = 0; deg < 360; deg += 5) {
      const angle = (deg * Math.PI) / 180 - Math.PI / 2;
      const cos = Math.cos(angle);
      const sin = Math.sin(angle);
      const isMajor = deg % 30 === 0;
      const isMedium = deg % 10 === 0;
      const innerR = isMajor ? cardR * 0.82 : isMedium ? cardR * 0.88 : cardR * 0.92;

      ctx.beginPath();
      ctx.moveTo(cos * innerR, sin * innerR);
      ctx.lineTo(cos * cardR, sin * cardR);
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = isMajor ? r * 0.015 : r * 0.006;
      ctx.stroke();

      if (isMajor) {
        const textR = cardR * 0.72;
        ctx.save();
        ctx.translate(cos * textR, sin * textR);
        ctx.rotate(angle + Math.PI / 2);
        ctx.fillStyle = '#fff';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        if (cardLabels[deg]) {
          ctx.font = `bold ${r * 0.12}px sans-serif`;
          ctx.fillText(cardLabels[deg], 0, 0);
        } else {
          ctx.font = `bold ${r * 0.09}px sans-serif`;
          ctx.fillText((deg / 10).toString(), 0, 0);
        }
        ctx.restore();
      }
    }
    ctx.restore();

    // Course pointer (lubber line at top)
    ctx.beginPath();
    ctx.moveTo(0, -r * 0.86);
    ctx.lineTo(-r * 0.035, -r * 0.92);
    ctx.lineTo(r * 0.035, -r * 0.92);
    ctx.closePath();
    ctx.fillStyle = '#ff8800';
    ctx.fill();

    // CDI dots (5 each side)
    const dotSpacing = r * 0.08;
    ctx.fillStyle = '#fff';
    for (let i = -5; i <= 5; i++) {
      if (i === 0) continue;
      ctx.beginPath();
      ctx.arc(i * dotSpacing, 0, r * 0.015, 0, Math.PI * 2);
      ctx.fill();
    }

    // Center ring (where CDI passes)
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.025, 0, Math.PI * 2);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.01;
    ctx.stroke();

    // CDI needle (vertical line, deflects left/right)
    // cdi: -5 to +5 dots
    const cdiOffset = Math.max(-5, Math.min(5, displayCdi)) * dotSpacing;
    ctx.beginPath();
    ctx.moveTo(cdiOffset, -r * 0.35);
    ctx.lineTo(cdiOffset, r * 0.35);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.025;
    ctx.lineCap = 'round';
    ctx.stroke();

    // TO/FROM flag
    ctx.save();
    const flagX = r * 0.35;
    const flagY = -r * 0.2;
    ctx.fillStyle = '#222';
    ctx.fillRect(flagX - r * 0.1, flagY - r * 0.07, r * 0.2, r * 0.14);
    ctx.strokeStyle = '#888';
    ctx.lineWidth = r * 0.008;
    ctx.strokeRect(flagX - r * 0.1, flagY - r * 0.07, r * 0.2, r * 0.14);

    ctx.fillStyle = '#fff';
    ctx.font = `bold ${r * 0.08}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    if (toFrom === 'TO') {
      // Draw upward triangle
      ctx.beginPath();
      ctx.moveTo(flagX, flagY - r * 0.04);
      ctx.lineTo(flagX - r * 0.04, flagY + r * 0.04);
      ctx.lineTo(flagX + r * 0.04, flagY + r * 0.04);
      ctx.closePath();
      ctx.fill();
      ctx.fillText('TO', flagX, flagY + r * 0.08);
    } else if (toFrom === 'FROM') {
      // Draw downward triangle
      ctx.beginPath();
      ctx.moveTo(flagX, flagY + r * 0.04);
      ctx.lineTo(flagX - r * 0.04, flagY - r * 0.04);
      ctx.lineTo(flagX + r * 0.04, flagY - r * 0.04);
      ctx.closePath();
      ctx.fill();
      ctx.fillText('FR', flagX, flagY + r * 0.08);
    } else {
      ctx.fillStyle = '#ff3333';
      ctx.fillText('OFF', flagX, flagY);
    }
    ctx.restore();

    // Glideslope dots (right side, vertical)
    ctx.fillStyle = '#fff';
    for (let i = -2; i <= 2; i++) {
      if (i === 0) continue;
      ctx.beginPath();
      ctx.arc(r * 0.6, i * dotSpacing * 1.5, r * 0.015, 0, Math.PI * 2);
      ctx.fill();
    }

    // GS center ring
    ctx.beginPath();
    ctx.arc(r * 0.6, 0, r * 0.025, 0, Math.PI * 2);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.01;
    ctx.stroke();

    // GS needle (horizontal bar, deflects up/down)
    const gsOffset = Math.max(-2.5, Math.min(2.5, displayGs)) * dotSpacing * 1.5;
    ctx.beginPath();
    ctx.moveTo(r * 0.45, gsOffset);
    ctx.lineTo(r * 0.75, gsOffset);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.025;
    ctx.lineCap = 'round';
    ctx.stroke();

    // Labels
    ctx.fillStyle = '#fff';
    ctx.font = `bold ${r * 0.07}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.fillText('VOR', -r * 0.35, -r * 0.2);
    ctx.font = `${r * 0.06}px sans-serif`;
    ctx.fillText('GS', r * 0.6, -r * 0.35);

    // OBS display
    ctx.fillStyle = '#222';
    ctx.fillRect(-r * 0.2, r * 0.52, r * 0.4, r * 0.14);
    ctx.strokeStyle = '#888';
    ctx.lineWidth = r * 0.008;
    ctx.strokeRect(-r * 0.2, r * 0.52, r * 0.4, r * 0.14);
    ctx.fillStyle = '#0f0';
    ctx.font = `bold ${r * 0.1}px monospace`;
    ctx.fillText(Math.round(displayObs).toString().padStart(3, '0') + '\u00B0', 0, r * 0.6);

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
