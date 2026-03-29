<script lang="ts">
  import { onMount } from 'svelte';

  let { pitch = 0, roll = 0 }: { pitch: number; roll: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayPitch = $state(0);
  let displayRoll = $state(0);
  let targetPitch = $state(0);
  let targetRoll = $state(0);
  let animId: number;

  $effect(() => {
    targetPitch = pitch;
    targetRoll = roll;
  });

  onMount(() => {
    function animate() {
      displayPitch += (targetPitch - displayPitch) * 0.15;
      displayRoll += (targetRoll - displayRoll) * 0.15;
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

    // Background (bezel area)
    const bg = ctx.createRadialGradient(0, 0, 0, 0, 0, r);
    bg.addColorStop(0, '#1a1a1a');
    bg.addColorStop(1, '#333');
    ctx.beginPath();
    ctx.arc(0, 0, r, 0, Math.PI * 2);
    ctx.fillStyle = bg;
    ctx.fill();

    // Clip to inner circle for horizon
    ctx.save();
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.85, 0, Math.PI * 2);
    ctx.clip();

    // Rotate for roll
    ctx.rotate((displayRoll * Math.PI) / 180);

    // Pitch offset (pixels per degree)
    const pitchPx = (r * 0.85) / 25; // ~25 degrees fills the view
    const pitchOffset = displayPitch * pitchPx;

    // Sky
    ctx.fillStyle = '#1a6dd4';
    ctx.fillRect(-r * 2, -r * 2 + pitchOffset, r * 4, r * 2);

    // Ground
    ctx.fillStyle = '#8B5E3C';
    ctx.fillRect(-r * 2, pitchOffset, r * 4, r * 2);

    // Horizon line
    ctx.beginPath();
    ctx.moveTo(-r * 2, pitchOffset);
    ctx.lineTo(r * 2, pitchOffset);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.015;
    ctx.stroke();

    // Pitch lines
    ctx.strokeStyle = '#fff';
    ctx.fillStyle = '#fff';
    ctx.font = `${r * 0.08}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';

    for (const deg of [5, 10, 15, 20]) {
      for (const sign of [-1, 1]) {
        const y = pitchOffset - sign * deg * pitchPx;
        const halfW = deg <= 5 ? r * 0.1 : deg <= 10 ? r * 0.2 : r * 0.3;
        ctx.beginPath();
        ctx.moveTo(-halfW, y);
        ctx.lineTo(halfW, y);
        ctx.lineWidth = deg % 10 === 0 ? r * 0.015 : r * 0.008;
        ctx.stroke();

        if (deg % 10 === 0) {
          ctx.fillText(deg.toString(), -halfW - r * 0.1, y);
          ctx.fillText(deg.toString(), halfW + r * 0.1, y);
        }
      }
    }

    ctx.restore(); // unclip

    // Bank angle indicators (fixed ring)
    ctx.save();
    const bankR = r * 0.87;
    const bankAngles = [0, 10, 20, 30, 60, 90];
    ctx.strokeStyle = '#fff';
    for (const ba of bankAngles) {
      for (const sign of ba === 0 ? [1] : [-1, 1]) {
        const angle = (-90 + sign * ba) * (Math.PI / 180);
        const isMajor = ba === 0 || ba === 30 || ba === 60 || ba === 90;
        const len = isMajor ? r * 0.1 : r * 0.06;
        ctx.beginPath();
        ctx.moveTo(Math.cos(angle) * bankR, Math.sin(angle) * bankR);
        ctx.lineTo(Math.cos(angle) * (bankR - len), Math.sin(angle) * (bankR - len));
        ctx.lineWidth = isMajor ? r * 0.02 : r * 0.012;
        ctx.stroke();
      }
    }

    // Bank pointer (triangle at top, rotates with roll)
    ctx.save();
    ctx.rotate((displayRoll * Math.PI) / 180);
    ctx.beginPath();
    ctx.moveTo(0, -bankR + r * 0.01);
    ctx.lineTo(-r * 0.035, -bankR + r * 0.08);
    ctx.lineTo(r * 0.035, -bankR + r * 0.08);
    ctx.closePath();
    ctx.fillStyle = '#fff';
    ctx.fill();
    ctx.restore();

    // Fixed sky pointer (triangle at top)
    ctx.beginPath();
    ctx.moveTo(0, -bankR);
    ctx.lineTo(-r * 0.03, -bankR - r * 0.06);
    ctx.lineTo(r * 0.03, -bankR - r * 0.06);
    ctx.closePath();
    ctx.fillStyle = '#ff8800';
    ctx.fill();
    ctx.restore();

    // Fixed aircraft symbol
    ctx.strokeStyle = '#ff8800';
    ctx.lineWidth = r * 0.035;
    ctx.lineCap = 'round';
    // Left wing
    ctx.beginPath();
    ctx.moveTo(-r * 0.4, 0);
    ctx.lineTo(-r * 0.15, 0);
    ctx.stroke();
    // Right wing
    ctx.beginPath();
    ctx.moveTo(r * 0.15, 0);
    ctx.lineTo(r * 0.4, 0);
    ctx.stroke();
    // Center dot
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.03, 0, Math.PI * 2);
    ctx.fillStyle = '#ff8800';
    ctx.fill();
    // Tails
    ctx.beginPath();
    ctx.moveTo(-r * 0.15, 0);
    ctx.lineTo(-r * 0.15, r * 0.06);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(r * 0.15, 0);
    ctx.lineTo(r * 0.15, r * 0.06);
    ctx.stroke();

    // Bezel
    ctx.lineWidth = r * 0.08;
    const bezel = ctx.createLinearGradient(-r, -r, r, r);
    bezel.addColorStop(0, '#666');
    bezel.addColorStop(0.5, '#aaa');
    bezel.addColorStop(1, '#666');
    ctx.beginPath();
    ctx.arc(0, 0, r, 0, Math.PI * 2);
    ctx.strokeStyle = bezel;
    ctx.stroke();

    // Glass effect
    const glass = ctx.createRadialGradient(0, -r * 0.3, 0, 0, 0, r);
    glass.addColorStop(0, 'rgba(255,255,255,0.03)');
    glass.addColorStop(1, 'rgba(0,0,0,0.12)');
    ctx.beginPath();
    ctx.arc(0, 0, r * 0.95, 0, Math.PI * 2);
    ctx.fillStyle = glass;
    ctx.fill();

    ctx.restore();
  }
</script>

<canvas bind:this={canvas} width="250" height="250" style="width:100%;height:100%;"></canvas>
