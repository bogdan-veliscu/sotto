<script lang="ts">
  import { onMount } from 'svelte';
  import { api, formatClock, formatStamp, isTauri } from '$lib/api';
  import type { Engine, SearchHit, Session, SessionDetail } from '$lib/types';

  let tauri = $state(false);
  let onboarded = $state(false);
  let sessions = $state<Session[]>([]);
  let engines = $state<Engine[]>([]);
  let selected = $state<SessionDetail | null>(null);
  let query = $state('');
  let hits = $state<SearchHit[]>([]);
  let err = $state('');
  let liveId = $state<string | null>(null);
  let liveStatus = $state('idle');
  let elapsed = $state(0);
  let timer: number | undefined;
  let settingsOpen = $state(false);
  let consentOpen = $state(false);
  let pendingId = $state<string | null>(null);
  let titleDraft = $state('');

  const consentText =
    'I am recording this conversation with Sotto. The audio stays on this computer. I have permission to record.';

  async function refresh() {
    sessions = await api.sessions();
    engines = await api.engines();
  }

  async function openSession(id: string) {
    selected = await api.session(id);
    titleDraft = selected.session.title;
  }

  async function boot() {
    err = '';
    tauri = isTauri();
    if (!tauri) return;
    const flag = await api.settingsGet('onboarding_complete');
    onboarded = flag === 'true';
    await refresh();
    if (sessions[0]) await openSession(sessions[0].id);
  }

  async function finishOnboarding() {
    await api.settingsSet('onboarding_complete', 'true');
    onboarded = true;
  }

  async function startRecord() {
    err = '';
    const session = await api.start(undefined, 'mixed');
    pendingId = session.id;
    consentOpen = true;
    await refresh();
  }

  async function acceptConsent() {
    if (!pendingId) return;
    await api.consent(pendingId);
    const begun = await api.begin(pendingId);
    liveId = begun.id;
    liveStatus = begun.status;
    consentOpen = false;
    elapsed = 0;
    window.clearInterval(timer);
    timer = window.setInterval(() => (elapsed += 1000), 1000);
    await openSession(begun.id);
    await refresh();
  }

  async function pause() {
    if (!liveId) return;
    const s = await api.pause(liveId);
    liveStatus = s.status;
    window.clearInterval(timer);
  }

  async function resume() {
    if (!liveId) return;
    const s = await api.resume(liveId);
    liveStatus = s.status;
    window.clearInterval(timer);
    timer = window.setInterval(() => (elapsed += 1000), 1000);
  }

  async function stopAndTranscribe() {
    if (!liveId) return;
    err = '';
    window.clearInterval(timer);
    await api.stopFixture(liveId);
    const detail = await api.transcribe(liveId);
    selected = detail;
    liveId = null;
    liveStatus = 'idle';
    elapsed = 0;
    await refresh();
  }

  async function runSearch() {
    if (!query.trim()) {
      hits = [];
      return;
    }
    hits = await api.search(query);
  }

  async function saveTitle() {
    if (!selected) return;
    await api.rename(selected.session.id, titleDraft);
    await refresh();
    await openSession(selected.session.id);
  }

  async function exportMd() {
    if (!selected) return;
    const md = await api.exportMd(selected.session.id);
    await navigator.clipboard.writeText(md);
  }

  async function removeSelected() {
    if (!selected) return;
    await api.deleteSession(selected.session.id);
    selected = null;
    await refresh();
  }

  onMount(() => {
    boot().catch((e) => (err = String(e)));
    return () => window.clearInterval(timer);
  });
</script>

<div class="desk">
  <header>
    <div class="brand">
      <span class="wordmark">Sotto</span>
      <span class="chip">on this Mac</span>
      <span class="chip ghost">no bot</span>
    </div>
    <form
      class="search"
      onsubmit={(e) => {
        e.preventDefault();
        runSearch().catch((e) => (err = String(e)));
      }}
    >
      <input bind:value={query} placeholder="Search transcripts on this Mac" />
    </form>
    <div class="rec">
      <span class="led" class:on={liveStatus === 'recording'}></span>
      <span class="mono clock">{formatClock(elapsed)}</span>
      {#if liveStatus === 'recording'}
        <button class="ghost" onclick={() => pause().catch((e) => (err = String(e)))}>Pause</button>
        <button class="danger" onclick={() => stopAndTranscribe().catch((e) => (err = String(e)))}>Stop</button>
      {:else if liveStatus === 'paused'}
        <button class="ghost" onclick={() => resume().catch((e) => (err = String(e)))}>Resume</button>
        <button class="danger" onclick={() => stopAndTranscribe().catch((e) => (err = String(e)))}>Stop</button>
      {:else}
        <button class="primary" onclick={() => startRecord().catch((e) => (err = String(e)))} disabled={!tauri}>
          Record
        </button>
      {/if}
      <button class="ghost" onclick={() => (settingsOpen = !settingsOpen)}>Models</button>
    </div>
  </header>

  {#if !tauri}
    <div class="banner">This is the desk shell. Run <span class="mono">make dev</span> to talk to the local store. <span class="mono">make demo</span> proves the privacy invariants without the UI.</div>
  {/if}
  {#if err}
    <div class="banner bad">{err}</div>
  {/if}

  <div class="body">
    <aside>
      <div class="aside-h">Sessions</div>
      {#if hits.length}
        <div class="hits">
          {#each hits as hit}
            <button class="row" onclick={() => openSession(hit.session_id).catch((e) => (err = String(e)))}>
              <strong>{hit.title}</strong>
              <span>{hit.snippet}</span>
            </button>
          {/each}
        </div>
      {/if}
      {#each sessions as s}
        <button
          class="row"
          class:active={selected?.session.id === s.id}
          onclick={() => openSession(s.id).catch((e) => (err = String(e)))}
        >
          <strong>{s.title}</strong>
          <em>{s.status} · {s.id}</em>
        </button>
      {:else}
        <p class="empty">No sessions yet. Record a fixture capture to seed the desk.</p>
      {/each}
    </aside>

    <main>
      {#if selected}
        <div class="title-row">
          <input class="title" bind:value={titleDraft} onchange={() => saveTitle().catch((e) => (err = String(e)))} />
          <button class="ghost" onclick={() => exportMd().catch((e) => (err = String(e)))}>Copy markdown</button>
          <button class="ghost" onclick={() => removeSelected().catch((e) => (err = String(e)))}>Delete</button>
        </div>
        <div class="meta mono">
          {selected.session.status}
          {#if selected.audio_encrypted}· encrypted{/if}
          {#if selected.session.model_id}· {selected.session.model_id}{/if}
        </div>
        {#if selected.summary}
          <section>
            <h2>Summary</h2>
            <p>{selected.summary}</p>
          </section>
        {/if}
        {#if selected.action_items}
          <section>
            <h2>Action items</h2>
            <pre>{selected.action_items}</pre>
          </section>
        {/if}
        <section>
          <h2>Transcript</h2>
          {#if selected.segments.length}
            {#each selected.segments as seg}
              <p class="seg"><span class="mono t">{formatStamp(seg.start_ms)}</span> {seg.text}</p>
            {/each}
          {:else}
            <p class="empty">No transcript yet. Stop a recording to run fixture replay.</p>
          {/if}
        </section>
      {:else}
        <div class="blank">
          <p class="wordmark">Quiet notes. Local audio.</p>
          <p>Sotto does not join the call. Wave 1 records a golden fixture, encrypts it, and transcribes offline.</p>
        </div>
      {/if}
    </main>
  </div>
</div>

{#if !onboarded && tauri}
  <div class="modal">
    <div class="card">
      <p class="wordmark">Sotto stays on this Mac.</p>
      <ul>
        <li>No meeting bot.</li>
        <li>Audio is encrypted at rest.</li>
        <li>Telemetry is off.</li>
        <li>Cloud engines stay off unless you turn them on.</li>
      </ul>
      <p class="fine">You are responsible for telling others when the law requires consent to record.</p>
      <button class="primary" onclick={() => finishOnboarding().catch((e) => (err = String(e)))}>Enter the desk</button>
    </div>
  </div>
{/if}

{#if consentOpen}
  <div class="modal">
    <div class="card">
      <p class="wordmark">Before you record</p>
      <blockquote>{consentText}</blockquote>
      <div class="actions">
        <button class="ghost" onclick={() => (consentOpen = false)}>Cancel</button>
        <button class="primary" onclick={() => acceptConsent().catch((e) => (err = String(e)))}>I can record</button>
      </div>
    </div>
  </div>
{/if}

{#if settingsOpen}
  <div
    class="modal"
    role="presentation"
    onclick={() => (settingsOpen = false)}
    onkeydown={(e) => e.key === 'Escape' && (settingsOpen = false)}
  >
    <div class="card wide" role="dialog" aria-modal="true" aria-label="Engines" tabindex="-1">
      <p class="wordmark">Engines</p>
      {#each engines as engine}
        <div class="engine">
          <strong>{engine.name}</strong>
          <span class="mono">{engine.mode} · {engine.install_state}</span>
          <p>{engine.notes}</p>
        </div>
      {/each}
      <p class="fine">Install is something you start. Demo never fetches weights. A failed checksum is discarded.</p>
      <button class="ghost" onclick={() => (settingsOpen = false)}>Close</button>
    </div>
  </div>
{/if}

<style>
  .desk { height: 100vh; display: flex; flex-direction: column; background:
    radial-gradient(1200px 500px at 80% -10%, #2a2018 0%, transparent 50%),
    var(--shell); }
  header { display: grid; grid-template-columns: auto 1fr auto; gap: 16px; align-items: center; padding: 14px 20px; border-bottom: 1px solid var(--line); }
  .brand { display: flex; gap: 8px; align-items: baseline; }
  .wordmark { font-size: 28px; }
  header .wordmark { font-size: 26px; }
  .chip { font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; background: var(--chip); padding: 4px 8px; color: var(--ok); }
  .chip.ghost { color: var(--mute); background: transparent; border: 1px solid var(--line); }
  .search input { width: 100%; background: var(--panel); border: 1px solid var(--line); color: var(--paper); padding: 10px 12px; }
  .rec { display: flex; gap: 8px; align-items: center; }
  .led { width: 10px; height: 10px; border-radius: 50%; background: #3a2a26; box-shadow: inset 0 0 0 1px #000; }
  .led.on { background: var(--led); box-shadow: 0 0 12px var(--led); animation: pulse 1.2s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.55; } }
  .clock { font-size: 18px; min-width: 4.5ch; }
  button { border: 0; background: var(--chip); color: var(--paper); padding: 8px 12px; }
  button.primary { background: var(--paper); color: var(--ink); font-weight: 600; }
  button.danger { background: var(--led); color: white; font-weight: 600; }
  button.ghost { background: transparent; border: 1px solid var(--line); }
  button:disabled { opacity: 0.4; cursor: not-allowed; }
  .banner { padding: 8px 20px; background: #241c14; color: var(--mute); font-size: 13px; }
  .banner.bad { background: #3a1814; color: #f3c0b6; }
  .body { flex: 1; display: grid; grid-template-columns: 280px 1fr; min-height: 0; }
  aside { border-right: 1px solid var(--line); overflow: auto; padding: 12px; }
  .aside-h { font-size: 11px; letter-spacing: 0.12em; text-transform: uppercase; color: var(--mute); margin-bottom: 8px; }
  .row { display: flex; flex-direction: column; gap: 4px; width: 100%; text-align: left; background: transparent; padding: 10px; border: 1px solid transparent; color: var(--paper); margin-bottom: 4px; }
  .row em { color: var(--mute); font-style: normal; font-size: 12px; font-family: 'IBM Plex Mono', monospace; }
  .row.active, .row:hover { background: var(--panel); border-color: var(--line); }
  main { overflow: auto; padding: 28px 36px 64px; }
  .title-row { display: flex; gap: 8px; align-items: center; }
  .title { flex: 1; background: transparent; border: 0; border-bottom: 1px solid var(--line); color: var(--paper); font-family: 'Fraunces', serif; font-size: 32px; padding: 4px 0; }
  .meta { color: var(--mute); margin: 8px 0 24px; font-size: 12px; }
  h2 { font-size: 11px; letter-spacing: 0.14em; text-transform: uppercase; color: var(--mute); font-weight: 600; }
  .seg { line-height: 1.55; margin: 0 0 10px; max-width: 62ch; }
  .t { color: var(--mute); margin-right: 10px; font-size: 12px; }
  .empty, .fine { color: var(--mute); }
  .blank { max-width: 36rem; padding-top: 12vh; }
  .blank .wordmark { font-size: 42px; display: block; margin-bottom: 12px; }
  .modal { position: fixed; inset: 0; background: rgba(10,8,6,0.72); display: grid; place-items: center; padding: 24px; }
  .card { background: var(--paper); color: var(--ink); padding: 28px; width: min(520px, 100%); }
  .card.wide { width: min(640px, 100%); }
  .card .wordmark { font-size: 32px; display: block; margin-bottom: 12px; }
  .card ul { padding-left: 1.1rem; }
  .card blockquote { background: #e7dcc6; padding: 12px 14px; margin: 0 0 16px; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; }
  .engine { border-top: 1px solid #d9ccb4; padding: 12px 0; }
  .engine span { display: block; color: #6d6458; font-size: 12px; }
  pre { white-space: pre-wrap; font-family: inherit; }
</style>
