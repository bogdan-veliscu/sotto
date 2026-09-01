<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { isTauri } from '$lib/api';

  type Hud = {
    led_on: boolean;
    paused: boolean;
    elapsed_ms: number;
    clock: string;
    caption: string;
  };

  let hud = $state<Hud>({
    led_on: false,
    paused: false,
    elapsed_ms: 0,
    clock: '00:00',
    caption: 'Sotto',
  });
  let timer: number | undefined;

  function tickFrom(ms: number, running: boolean) {
    window.clearInterval(timer);
    hud.elapsed_ms = ms;
    const stamp = () => {
      const total = Math.floor(hud.elapsed_ms / 1000);
      hud.clock = `${String(Math.floor(total / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}`;
    };
    stamp();
    if (!running) return;
    timer = window.setInterval(() => {
      hud.elapsed_ms += 1000;
      stamp();
    }, 1000);
  }

  onMount(() => {
    if (!isTauri()) return;
    const unlisten = listen<Hud>('sotto://hud', (event) => {
      hud = event.payload;
      tickFrom(event.payload.elapsed_ms, event.payload.led_on);
    });
    return () => {
      window.clearInterval(timer);
      unlisten.then((fn) => fn());
    };
  });
</script>

<div class="hud" class:live={hud.led_on} class:paused={hud.paused}>
  <span class="led" class:on={hud.led_on}></span>
  <span class="mono clock">{hud.clock}</span>
  <span class="cap">{hud.caption}</span>
</div>

<style>
  :global(html, body) { background: transparent; }
  .hud {
    display: flex; align-items: center; gap: 8px;
    height: 44px; padding: 0 14px;
    background: rgba(20, 17, 14, 0.88);
    color: #f3ead8; border: 1px solid #2a241e;
    border-radius: 999px;
    -webkit-app-region: drag;
  }
  .led { width: 9px; height: 9px; border-radius: 50%; background: #3a2a26; }
  .led.on { background: #d23a22; box-shadow: 0 0 10px #d23a22; }
  .clock { font-size: 14px; min-width: 4.5ch; }
  .cap { font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; color: #8a7e6e; }
</style>
