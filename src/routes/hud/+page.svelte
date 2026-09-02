<script lang="ts">
  import { onMount } from 'svelte';
  import { emit, listen } from '@tauri-apps/api/event';
  import { api, isTauri } from '$lib/api';

  type Hud = {
    led_on: boolean;
    paused: boolean;
    elapsed_ms: number;
    clock: string;
    caption: string;
    status_label?: string;
    source?: string;
    source_label?: string;
    title?: string;
    session_id?: string;
    level?: number;
  };

  let hud = $state<Hud>({
    led_on: false,
    paused: false,
    elapsed_ms: 0,
    clock: '00:00',
    caption: 'Sotto',
    status_label: 'Sotto',
    source_label: '',
    title: '',
    session_id: '',
    level: 0,
  });
  let quietFor = $state(0);
  let timer: number | undefined;
  let meterTimer: number | undefined;
  const bars = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

  function stamp(ms: number) {
    const total = Math.max(0, Math.floor(ms / 1000));
    hud.clock = `${String(Math.floor(total / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}`;
  }

  function tickFrom(ms: number, running: boolean, keepClock: boolean) {
    window.clearInterval(timer);
    if (!keepClock) hud.elapsed_ms = ms;
    stamp(hud.elapsed_ms);
    if (!running) return;
    timer = window.setInterval(() => {
      hud.elapsed_ms += 1000;
      stamp(hud.elapsed_ms);
    }, 1000);
  }

  function apply(next: Hud) {
    const keepClock = (next.paused || next.led_on) && next.elapsed_ms === 0 && hud.elapsed_ms > 0;
    hud = {
      ...hud,
      ...next,
      elapsed_ms: keepClock ? hud.elapsed_ms : next.elapsed_ms,
      clock: keepClock ? hud.clock : next.clock,
      level: next.paused ? (hud.level ?? 0) : (next.level ?? hud.level ?? 0),
    };
    tickFrom(hud.elapsed_ms, next.led_on, keepClock);
    if (!next.led_on) quietFor = 0;
  }

  async function pulse() {
    if (!isTauri() || (!hud.led_on && !hud.paused)) return;
    try {
      const next = await api.hudTick();
      const level = next.level ?? 0;
      if (!next.paused) hud.level = level;
      if (next.session_id) hud.session_id = next.session_id;
      if (next.source_label) hud.source_label = next.source_label;
      if (next.title) hud.title = next.title;
      if (next.status_label) hud.status_label = next.status_label;
      hud.paused = next.paused;
      hud.led_on = next.led_on;
      if (!next.led_on && !next.paused) {
        tickFrom(0, false, false);
        quietFor = 0;
        return;
      }
      if (level < 4 && hud.led_on) quietFor += 1;
      else quietFor = 0;
    } catch {
      /* HUD can sit idle while the desk is down */
    }
  }

  async function togglePause() {
    const id = hud.session_id;
    if (!id) return;
    try {
      if (hud.paused) await api.resume(id);
      else await api.pause(id);
    } catch {
      /* desk shows the recoverable error */
    }
  }

  async function stopLive() {
    const id = hud.session_id;
    if (!id) return;
    try {
      await api.stop(id);
      await emit('sotto://hud-stopped', { sessionId: id });
    } catch {
      /* desk shows the recoverable error */
    }
  }

  function barPx(i: number, level: number, silent: boolean) {
    if (level <= i * 8) return 3;
    if (silent) return 5;
    return 6 + (i % 4) * 3;
  }

  const quiet = $derived(hud.led_on && quietFor >= 16);
  const status = $derived(
    hud.status_label || (hud.paused ? 'Paused' : hud.led_on ? 'Rec' : 'Sotto'),
  );
  const source = $derived(quiet ? 'No signal' : hud.source_label || 'Local');
  const tip = $derived(
    [hud.title, hud.caption].filter((part) => part && part !== 'Sotto').join(' · ') || 'Sotto',
  );

  onMount(() => {
    if (!isTauri()) return;
    const unlisten = listen<Hud>('sotto://hud', (event) => apply(event.payload));
    meterTimer = window.setInterval(() => pulse().catch(() => undefined), 120);
    return () => {
      window.clearInterval(timer);
      window.clearInterval(meterTimer);
      unlisten.then((fn) => fn());
    };
  });
</script>

<div
  class="island"
  class:live={hud.led_on}
  class:paused={hud.paused}
  class:quiet
  title={tip}
>
  <div class="state">
    <span class="led" class:on={hud.led_on} class:hold={hud.paused}></span>
    <span class="label">{status}</span>
  </div>
  <span class="mono clock">{hud.clock}</span>
  <div class="meter" aria-hidden="true">
    {#each bars as i}
      {@const on = (hud.level ?? 0) > i * 8}
      <span
        class="bar"
        class:on
        class:hot={on && i >= 9}
        class:clip={on && i >= 11 && (hud.level ?? 0) > 92}
        style="height: {barPx(i, hud.level ?? 0, quiet)}px"
      ></span>
    {/each}
  </div>
  <span class="source">{source}</span>
  <div class="hit">
    <button class="act" onclick={togglePause} aria-label={hud.paused ? 'Resume' : 'Pause'}>
      {#if hud.paused}
        <span class="play"></span>
      {:else}
        <span class="pause"></span>
      {/if}
    </button>
    <button class="act stop" onclick={stopLive} aria-label="Stop">
      <span class="sq"></span>
    </button>
  </div>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
    overflow: hidden;
  }
  .island {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 58px;
    padding: 0 6px 0 12px;
    background:
      linear-gradient(180deg, rgba(42, 36, 30, 0.98) 0%, rgba(16, 13, 11, 0.99) 100%);
    color: #f3ead8;
    border: 1px solid rgba(243, 234, 216, 0.18);
    border-radius: 999px;
    box-shadow:
      0 12px 28px rgba(0, 0, 0, 0.45),
      inset 0 1px 0 rgba(255, 255, 255, 0.1);
    -webkit-app-region: drag;
    user-select: none;
    font-family: 'IBM Plex Sans', sans-serif;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 7px;
    min-width: 4.2rem;
  }
  .led {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #4a3f38;
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.4);
    flex: 0 0 auto;
  }
  .led.on {
    background: #ff4d38;
    box-shadow: 0 0 0 3px rgba(210, 58, 34, 0.22), 0 0 12px #d23a22;
    animation: breathe 1.4s ease-in-out infinite;
  }
  .led.hold {
    background: #e2b43a;
    box-shadow: 0 0 0 3px rgba(201, 162, 39, 0.2);
    animation: none;
  }
  @keyframes breathe {
    50% {
      opacity: 0.55;
    }
  }
  .label {
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: #f3ead8;
  }
  .paused .label {
    color: #f0d48a;
  }
  .clock {
    font-size: 15px;
    font-variant-numeric: tabular-nums;
    min-width: 4.6ch;
    letter-spacing: 0.02em;
  }
  .meter {
    display: flex;
    align-items: flex-end;
    gap: 2px;
    height: 18px;
    width: 58px;
    flex: 0 0 auto;
    padding-bottom: 1px;
  }
  .bar {
    width: 3px;
    border-radius: 1px;
    background: #3a322c;
    transition: height 80ms linear, background 80ms linear;
  }
  .bar.on {
    background: #c9d7b4;
  }
  .bar.hot.on {
    background: #e2b43a;
  }
  .bar.clip.on {
    background: #ff4d38;
  }
  .quiet .bar.on {
    background: #8a7e6e;
  }
  .source {
    font-size: 11px;
    font-weight: 500;
    color: #d8ccb8;
    max-width: 4.8rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .quiet .source {
    color: #e2b43a;
  }
  .hit {
    display: flex;
    gap: 4px;
    margin-left: auto;
    padding-left: 4px;
    border-left: 1px solid rgba(243, 234, 216, 0.12);
    -webkit-app-region: no-drag;
  }
  .act {
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 999px;
    background: rgba(243, 234, 216, 0.08);
    color: #f3ead8;
    display: grid;
    place-items: center;
    padding: 0;
  }
  .act:hover {
    background: rgba(243, 234, 216, 0.16);
  }
  .pause {
    width: 8px;
    height: 10px;
    border-left: 2.5px solid #f3ead8;
    border-right: 2.5px solid #f3ead8;
  }
  .play {
    width: 0;
    height: 0;
    border-top: 5px solid transparent;
    border-bottom: 5px solid transparent;
    border-left: 8px solid #f3ead8;
    margin-left: 2px;
  }
  .sq {
    width: 8px;
    height: 8px;
    border-radius: 1px;
    background: #ff4d38;
  }
</style>
