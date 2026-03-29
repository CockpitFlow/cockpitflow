<script lang="ts">
  import { onMount } from 'svelte';

  let { turn_rate = 0, slip = 0 }: { turn_rate: number; slip: number } = $props();

  let canvas: HTMLCanvasElement;
  let displayTurn = $state(0);
  let displaySlip = $state(0);
  let targetTurn = $state(0);
  let targetSlip = $state(0);
  let animId: number;

  $effect(() => {
    targetTurn = turn_rate;
    targetSlip = slip;
  });

  onMount(() => {
    function animate() {
      displayTurn += (targetTurn - displayTurn) * 0.15;
      displaySlip += (targetSlip - displaySlip) * 0.15;
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

    // Labels
    ctx.fillStyle = '#fff';
    ctx.font = `bold ${r * 0.09}px sans-serif`;
    ctx.textAlign = 'center';
    ctx.fillText('TURN COORDINATOR', 0, -r * 0.6);
    ctx.font = `${r * 0.065}px sans-serif`;
    ctx.fillText('2 MIN', 0, r * 0.2);
    ctx.fillText('NO PITCH', 0, r * 0.32);
    ctx.fillText('INFORMATION', 0, r * 0.42);

    // Standard rate turn marks (L and R)
    // Standard rate = ~3 deg/sec. Mark at about +-20 degrees tilt on the display
    const markAngle = 20 * (Math.PI / 180);
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.025;
    for (const sign of [-1, 1]) {
      const a = sign * markAngle;
      const mx = Math.sin(a) * r * 0.5;
      const my = -Math.cos(a) * r * 0.05 - r * 0.1;
      ctx.beginPath();
      ctx.moveTo(mx - r * 0.08 * Math.cos(a), my - r * 0.08 * Math.sin(-a));
      ctx.lineTo(mx + r * 0.08 * Math.cos(a), my + r * 0.08 * Math.sin(-a));
      ctx.stroke();
    }

    // "L" and "R" labels
    ctx.font = `bold ${r * 0.1}px sans-serif`;
    ctx.fillStyle = '#fff';
    ctx.fillText('L', -r * 0.65, -r * 0.1);
    ctx.fillText('R', r * 0.65, -r * 0.1);

    // Miniature airplane (tilts based on turn rate)
    // Map turn_rate: standard rate ~3 deg/s -> ~20 deg tilt
    const tiltAngle = Math.max(-45, Math.min(45, displayTurn * (20 / 3))) * (Math.PI / 180);

    ctx.save();
    ctx.rotate(tiltAngle);

    // Aircraft body
    ctx.shadowColor = 'rgba(0,0,0,0.4)';
    ctx.shadowBlur = r * 0.04;
    ctx.shadowOffsetY = r * 0.02;

    ctx.fillStyle = '#fff';
    ctx.strokeStyle = '#111';
    ctx.lineWidth = r * 0.01;

    // Fuselage
    ctx.beginPath();
    ctx.ellipse(0, -r * 0.05, r * 0.025, r * 0.15, 0, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();

    // Wings
    ctx.beginPath();
    ctx.moveTo(-r * 0.45, -r * 0.03);
    ctx.lineTo(r * 0.45, -r * 0.03);
    ctx.lineTo(r * 0.45, r * 0.0);
    ctx.lineTo(-r * 0.45, r * 0.0);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    // Tail
    ctx.beginPath();
    ctx.moveTo(-r * 0.12, -r * 0.17);
    ctx.lineTo(r * 0.12, -r * 0.17);
    ctx.lineTo(r * 0.12, -r * 0.15);
    ctx.lineTo(-r * 0.12, -r * 0.15);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    ctx.restore();

    // Slip/skid ball tube
    const tubeY = r * 0.6;
    const tubeW = r * 0.5;
    const tubeH = r * 0.1;
    const tubeCurve = r * 0.8; // radius of curved tube

    // Draw tube
    ctx.beginPath();
    ctx.arc(0, tubeY + tubeCurve - tubeH, tubeCurve,
      Math.PI + Math.asin(tubeW / tubeCurve), -Math.asin(tubeW / tubeCurve));
    ctx.strokeStyle = '#888';
    ctx.lineWidth = tubeH;
    ctx.stroke();

    // Center marks on tube
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = r * 0.015;
    ctx.beginPath();
    ctx.moveTo(-r * 0.06, tubeY - tubeH * 0.5);
    ctx.lineTo(-r * 0.06, tubeY + tubeH * 0.5);
    ctx.stroke();
    ctx.beginPath();
    ctx.moveTo(r * 0.06, tubeY - tubeH * 0.5);
    ctx.lineTo(r * 0.06, tubeY + tubeH * 0.5);
    ctx.stroke();

    // Ball - slip maps to horizontal offset
    const ballOffset = Math.max(-1, Math.min(1, displaySlip)) * r * 0.2;
    const ballGrad = ctx.createRadialGradient(ballOffset - r * 0.01, tubeY - r * 0.01, 0, ballOffset, tubeY, r * 0.04);
    ballGrad.addColorStop(0, '#222');
    ballGrad.addColorStop(0.7, '#111');
    ballGrad.addColorStop(1, '#000');
    ctx.beginPath();
    ctx.arc(ballOffset, tubeY, r * 0.04, 0, Math.PI * 2);
    ctx.fillStyle = ballGrad;
    ctx.fill();
    ctx.strokeStyle = '#555';
    ctx.lineWidth = r * 0.005;
    ctx.stroke();

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
