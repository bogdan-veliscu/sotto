<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { api, formatClock, formatStamp, isTauri } from '$lib/api';
  import type {
    Engine,
    PrivacySettings,
    RecoverableCapture,
    SearchHit,
    Session,
    SessionDetail,
  } from '$lib/types';
  import { open, save } from '@tauri-apps/plugin-dialog';

  let tauri = $state(false);
  let onboarded = $state(false);
  let sessions = $state<Session[]>([]);
  let recoveries = $state<RecoverableCapture[]>([]);
  let discardId = $state<string | null>(null);
  let engines = $state<Engine[]>([]);
  let selected = $state<SessionDetail | null>(null);
  let query = $state('');
  let titleFilter = $state('');
  let fromDay = $state('');
  let toDay = $state('');
  let tagFilter = $state('');
  let tagDraft = $state('');
  let hits = $state<SearchHit[]>([]);
  let err = $state('');
  let busy = $state('');
  let download = $state<{
    active: boolean;
    failed: boolean;
    variant: string;
    file: string;
    file_index: number;
    file_count: number;
    received: number;
    percent: number;
    message: string;
  } | null>(null);
  let liveId = $state<string | null>(null);
  let liveStatus = $state('idle');
  let elapsed = $state(0);
  let timer: number | undefined;
  let settingsOpen = $state(false);
  let consentOpen = $state(false);
  let deleteAllOpen = $state(false);
  let pendingId = $state<string | null>(null);
  let titleDraft = $state('');
  let defaultModel = $state('fixture-replay');
  let login = $state({ backend: 'unsupported', requested: false, applied: false });
  let hotkeyShortcut = $state('CommandOrControl+Shift+Space');
  let hotkeyMode = $state('toggle');
  let meetingDetect = $state(false);
  let tapStatus = $state('unsupported');
  let parakeetStatus = $state('not-built');
  let captureSource = $state('mic');
  let sourceHint = $state('Microphone access is required. Consent is still required.');
  let meetingAskOpen = $state(false);
  let meetingCopy = $state('');
  let ignoredKinds = $state<string[]>([]);
  let privacy = $state<PrivacySettings>({
    telemetry: 'off',
    cloud_mode: 'off',
    retention_days: '0',
  });

  const consentText =
    'I am recording this conversation with Sotto. The audio stays on this computer. I have permission to record.';

  function fail(e: unknown) {
    err = e instanceof Error ? e.message : String(e);
  }

  async function refresh() {
    sessions = await api.sessions();
    engines = await api.engines();
    recoveries = await api.recoveries();
  }

  async function recoverCapture(sessionId: string) {
    err = '';
    busy = 'Recovering…';
    try {
      const session = await api.recoverLive(sessionId);
      await refresh();
      await openSession(session.id);
    } finally {
      busy = '';
    }
  }

  async function confirmDiscardLive() {
    if (!discardId) return;
    err = '';
    const id = discardId;
    discardId = null;
    await api.discardLive(id);
    await refresh();
    if (selected?.session.id === id) await openSession(id);
  }

  async function openSession(id: string) {
    selected = await api.session(id);
    titleDraft = selected.session.title;
    tagDraft = selected.tags.join(', ');
  }

  async function boot() {
    err = '';
    tauri = isTauri();
    if (!tauri) return;
    const flag = await api.settingsGet('onboarding_complete');
    onboarded = flag === 'true';
    defaultModel = (await api.settingsGet('default_model')) ?? 'fixture-replay';
    const savedSource = await api.settingsGet('capture_source');
    if (savedSource === 'mic' || savedSource === 'system' || savedSource === 'mixed') {
      captureSource = savedSource;
    }
    await persistSource();
    await refresh();
    await preferLiveDefault();
    if (sessions[0]) await openSession(sessions[0].id);
  }

  async function finishOnboarding() {
    await api.settingsSet('onboarding_complete', 'true');
    onboarded = true;
  }

  async function persistSource() {
    if (captureSource !== 'mic' && captureSource !== 'system' && captureSource !== 'mixed') {
      captureSource = 'mic';
    }
    if (!tauri) return;
    await api.settingsSet('capture_source', captureSource);
    sourceHint = await api.sourceHint(captureSource);
  }

  async function startRecord() {
    err = '';
    await persistSource();
    const session = await api.start(undefined, captureSource);
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

  async function onHotkey(payload: { mode: string; state: string }) {
    if (busy) return;
    const ptt = payload.mode === 'ptt';
    if (ptt) {
      if (payload.state === 'pressed') {
        if (liveStatus === 'paused') await resume();
        else if (liveStatus === 'idle' && !consentOpen) await startRecord();
      } else if (payload.state === 'released' && liveStatus === 'recording') {
        await pause();
      }
      return;
    }
    if (payload.state !== 'pressed') return;
    if (liveStatus === 'recording' || liveStatus === 'paused') await stopAndTranscribe();
    else if (!consentOpen) await startRecord();
  }

  async function scanMeetings() {
    if (!tauri || !onboarded || liveStatus !== 'idle' || consentOpen || meetingAskOpen || settingsOpen) {
      return;
    }
    const report = await api.meetingGet();
    meetingDetect = report.enabled;
    if (!report.should_prompt) return;
    const kinds = report.detected.map((d) => d.kind);
    if (kinds.every((k) => ignoredKinds.includes(k))) return;
    meetingCopy = report.prompt;
    meetingAskOpen = true;
  }

  function dismissMeetingAsk() {
    meetingAskOpen = false;
    api
      .meetingGet()
      .then((report) => {
        ignoredKinds = Array.from(new Set([...ignoredKinds, ...report.detected.map((d) => d.kind)]));
      })
      .catch(fail);
  }

  async function acceptMeetingAsk() {
    meetingAskOpen = false;
    await startRecord();
  }

  const ENGINE_ORDER = [
    'apple-speech-ondevice',
    'parakeet-tdt-0.6b-v3',
    'whisper-large-v3-turbo',
    'fixture-replay',
  ];

  const listedEngines = $derived(
    [...engines].sort((a, b) => {
      const ia = ENGINE_ORDER.indexOf(a.id);
      const ib = ENGINE_ORDER.indexOf(b.id);
      return (ia < 0 ? 99 : ia) - (ib < 0 ? 99 : ib);
    }),
  );

  function engineTone(engine: Engine): 'ok' | 'wait' | 'demo' {
    if (engine.id === 'fixture-replay') return 'demo';
    if (engine.live_ready) return 'ok';
    return 'wait';
  }

  function engineStatus(engine: Engine): string {
    if (engine.id === 'fixture-replay') return 'Demo only';
    if (engine.live_ready) return 'Ready for live';
    if (engine.id === 'apple-speech-ondevice') return 'Unavailable';
    return 'Needs weights';
  }

  function engineBlurb(engine: Engine): string {
    switch (engine.id) {
      case 'apple-speech-ondevice':
        return engine.live_ready
          ? 'Transcribes on this Mac. No download. Audio is not sent to Apple servers.'
          : 'Needs on-device Apple speech on this Mac. Audio is not sent to Apple servers.';
      case 'parakeet-tdt-0.6b-v3':
        return engine.live_ready
          ? 'Local TDT weights are installed.'
          : 'Download INT8 (~670 MB) or import a folder. Demo never fetches.';
      case 'whisper-large-v3-turbo':
        return 'Import a local ggml file. Not the Whisper API.';
      case 'fixture-replay':
        return 'Locked golden transcript for make demo. Will not transcribe a live meeting.';
      default:
        return engine.notes;
    }
  }

  function liveEngines() {
    return engines.filter((e) => e.live_ready && e.id !== 'fixture-replay');
  }

  function preferredLiveEngine() {
    const live = liveEngines();
    const chosen = live.find((e) => e.id === defaultModel);
    if (chosen) return chosen;
    return live.find((e) => e.id === 'apple-speech-ondevice') ?? live[0];
  }

  function liveTranscribeModel(): string | undefined {
    return preferredLiveEngine()?.id;
  }

  async function preferLiveDefault() {
    if (defaultModel !== 'fixture-replay') return;
    const live = preferredLiveEngine();
    if (!live) return;
    await setDefaultModel(live.id);
  }

  const appleEngine = $derived(engines.find((e) => e.id === 'apple-speech-ondevice'));
  const parakeetEngine = $derived(engines.find((e) => e.id === 'parakeet-tdt-0.6b-v3'));
  const liveEngineLabel = $derived(
    preferredLiveEngine()?.name ?? appleEngine?.name ?? 'Choose an engine',
  );

  async function stopAndTranscribe() {
    if (!liveId) return;
    const id = liveId;
    err = '';
    busy = 'Encrypting on this Mac…';
    window.clearInterval(timer);
    try {
      await api.stop(id);
      await transcribeAfterStop(id);
    } finally {
      liveId = null;
      liveStatus = 'idle';
      elapsed = 0;
      busy = '';
    }
    await refresh();
  }

  async function transcribeAfterStop(id: string) {
    busy = 'Transcribing on this Mac…';
    try {
      const detail = await api.transcribe(id, liveTranscribeModel());
      selected = detail;
      titleDraft = detail.session.title;
      tagDraft = detail.tags.join(', ');
    } catch (e) {
      fail(e);
      await openSession(id);
    }
  }

  async function transcribeSelected() {
    if (!selected) return;
    err = '';
    busy = 'Transcribing on this Mac…';
    try {
      const detail = await api.transcribe(selected.session.id, liveTranscribeModel());
      selected = detail;
      titleDraft = detail.session.title;
      tagDraft = detail.tags.join(', ');
    } catch (e) {
      fail(e);
    } finally {
      busy = '';
      await refresh();
    }
  }

  function dayBound(iso: string, end: boolean): string | undefined {
    if (!iso) return undefined;
    const suffix = end ? 'T23:59:59' : 'T00:00:00';
    const ms = new Date(`${iso}${suffix}`).getTime();
    if (Number.isNaN(ms)) return undefined;
    return String(Math.floor(ms / 1000));
  }

  async function runSearch() {
    const filters = {
      title: titleFilter.trim() || undefined,
      createdFrom: dayBound(fromDay, false),
      createdTo: dayBound(toDay, true),
      tag: tagFilter.trim() || undefined,
    };
    if (!query.trim() && !filters.title && !filters.createdFrom && !filters.createdTo && !filters.tag) {
      hits = [];
      return;
    }
    hits = await api.search(query, filters);
  }

  async function saveTags() {
    if (!selected) return;
    const tags = tagDraft
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);
    const saved = await api.setTags(selected.session.id, tags);
    selected = { ...selected, tags: saved };
    tagDraft = saved.join(', ');
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

  async function saveMd() {
    if (!selected) return;
    const dest = await save({
      defaultPath: `${selected.session.title.replace(/[/\\]/g, '-')}.md`,
      filters: [{ name: 'Markdown', extensions: ['md'] }],
    });
    if (!dest) return;
    await api.exportFile(selected.session.id, dest);
  }

  async function openSettings() {
    settingsOpen = true;
    if (!tauri) return;
    privacy = await api.privacy();
    engines = await api.engines();
    defaultModel = (await api.settingsGet('default_model')) ?? 'fixture-replay';
    login = await api.loginGet();
    const hotkey = await api.hotkeyGet();
    hotkeyShortcut = hotkey.shortcut;
    hotkeyMode = hotkey.mode;
    const meeting = await api.meetingGet();
    meetingDetect = meeting.enabled;
    tapStatus = await api.tapStatus();
    parakeetStatus = await api.parakeetRuntime();
    await persistSource();
  }

  async function saveHotkey() {
    const saved = await api.hotkeySet(hotkeyShortcut, hotkeyMode);
    hotkeyShortcut = saved.shortcut;
    hotkeyMode = saved.mode;
  }

  async function setPrivacy(key: 'telemetry' | 'cloud_mode', value: string) {
    await api.settingsSet(key, value);
    privacy = await api.privacy();
  }

  async function setDefaultModel(id: string) {
    defaultModel = id;
    await api.settingsSet('default_model', id);
  }

  async function importEngine(engineId: string) {
    const directory = engineId === 'parakeet-tdt-0.6b-v3';
    const picked = await open({
      multiple: false,
      directory,
      title: directory ? 'Choose Parakeet TDT folder' : 'Choose Whisper ggml file',
    });
    if (!picked || Array.isArray(picked)) return;
    busy = 'Importing local weights…';
    try {
      await api.importModel(engineId, picked);
      await refresh();
      engines = await api.engines();
    } finally {
      busy = '';
    }
  }

  async function downloadParakeet(variant: 'int8' | 'fp32') {
    settingsOpen = true;
    err = '';
    download = {
      active: true,
      failed: false,
      variant,
      file: '',
      file_index: 0,
      file_count: variant === 'int8' ? 3 : 4,
      received: 0,
      percent: 0,
      message:
        variant === 'int8'
          ? 'Starting Parakeet INT8 (~670 MB)…'
          : 'Starting Parakeet FP32 (~2.5 GB)…',
    };
    busy = download.message;
    try {
      await api.downloadParakeet(variant);
      await refresh();
      engines = await api.engines();
      await setDefaultModel('parakeet-tdt-0.6b-v3');
      download = {
        active: false,
        failed: false,
        variant,
        file: '',
        file_index: download?.file_count ?? 0,
        file_count: download?.file_count ?? 0,
        received: download?.received ?? 0,
        percent: 100,
        message: 'Parakeet is ready. Using it for live recordings.',
      };
      busy = download.message;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      fail(e);
      download = {
        active: false,
        failed: true,
        variant,
        file: download?.file ?? '',
        file_index: download?.file_index ?? 0,
        file_count: download?.file_count ?? 0,
        received: download?.received ?? 0,
        percent: download?.percent ?? 0,
        message,
      };
    } finally {
      if (!download?.failed) {
        window.setTimeout(() => {
          if (download && !download.active && !download.failed) busy = '';
        }, 2500);
      } else {
        busy = '';
      }
    }
  }

  async function removeEngine(engineId: string) {
    await api.deleteModel(engineId);
    engines = await api.engines();
  }

  async function confirmDeleteAll() {
    await api.deleteAll();
    selected = null;
    hits = [];
    deleteAllOpen = false;
    settingsOpen = false;
    await refresh();
  }

  async function removeSelected() {
    if (!selected) return;
    await api.deleteSession(selected.session.id);
    selected = null;
    await refresh();
  }

  onMount(() => {
    boot().catch(fail);
    let unlisten: Promise<() => void> | undefined;
    let unlistenHud: Promise<() => void> | undefined;
    let unlistenDl: Promise<() => void> | undefined;
    let scanTimer: number | undefined;
    if (isTauri()) {
      unlisten = listen<{ mode: string; state: string }>('sotto://hotkey', (event) => {
        onHotkey(event.payload).catch(fail);
      });
      unlistenHud = listen<{ sessionId: string }>('sotto://hud-stopped', (event) => {
        const id = event.payload.sessionId;
        liveId = null;
        liveStatus = 'idle';
        window.clearInterval(timer);
        elapsed = 0;
        busy = 'Transcribing on this Mac…';
        transcribeAfterStop(id)
          .catch(fail)
          .finally(() => {
            busy = '';
            refresh().catch(fail);
          });
      });
      unlistenDl = listen<{
        phase: string;
        variant: string;
        file: string;
        file_index: number;
        file_count: number;
        received: number;
        percent: number;
        message: string;
      }>('sotto://model-download', (event) => {
        const next = event.payload;
        download = {
          active: next.phase !== 'done' && next.phase !== 'error',
          failed: next.phase === 'error',
          variant: next.variant,
          file: next.file,
          file_index: next.file_index,
          file_count: next.file_count,
          received: next.received,
          percent: next.percent,
          message: next.message,
        };
        if (download.active || download.failed) busy = next.message;
      });
      scanTimer = window.setInterval(() => scanMeetings().catch(fail), 15000);
      scanMeetings().catch(fail);
    }
    return () => {
      window.clearInterval(timer);
      window.clearInterval(scanTimer);
      unlisten?.then((fn) => fn());
      unlistenHud?.then((fn) => fn());
      unlistenDl?.then((fn) => fn());
    };
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
        runSearch().catch(fail);
      }}
    >
      <input bind:value={query} placeholder="Search transcripts on this Mac" />
    </form>
    <div class="rec">
      <span class="led" class:on={liveStatus === 'recording'}></span>
      <span class="mono clock">{formatClock(elapsed)}</span>
      {#if liveStatus === 'recording'}
        <button class="ghost" onclick={() => pause().catch(fail)}>Pause</button>
        <button class="danger" onclick={() => stopAndTranscribe().catch(fail)}>Stop</button>
      {:else if liveStatus === 'paused'}
        <button class="ghost" onclick={() => resume().catch(fail)}>Resume</button>
        <button class="danger" onclick={() => stopAndTranscribe().catch(fail)}>Stop</button>
      {:else}
        <select
          class="source-pick"
          bind:value={captureSource}
          onchange={() => persistSource().catch(fail)}
          disabled={!tauri || !!busy}
          aria-label="Capture source"
        >
          <option value="mic">Microphone</option>
          <option value="system">What you hear</option>
          <option value="mixed">Mixed</option>
        </select>
        <button class="primary" onclick={() => startRecord().catch(fail)} disabled={!tauri || !!busy}>
          Record
        </button>
      {/if}
      <select
        class="source-pick"
        value={liveTranscribeModel() ?? defaultModel}
        onchange={(e) => setDefaultModel((e.currentTarget as HTMLSelectElement).value).catch(fail)}
        disabled={!tauri || !!busy}
        aria-label="Transcription engine"
      >
        {#each listedEngines.filter((e) => e.id !== 'fixture-replay') as engine}
          <option value={engine.id} disabled={!engine.live_ready}>{engine.name}</option>
        {/each}
      </select>
      {#if parakeetEngine && (!parakeetEngine.live_ready || download?.active)}
        <button class="ghost" disabled={!tauri || !!busy} onclick={() => downloadParakeet('int8').catch(fail)}>
          {#if download?.active}
            Downloading {download.percent}%
          {:else}
            Download Parakeet
          {/if}
        </button>
      {/if}
      <button class="ghost" onclick={() => openSettings().catch(fail)}>Models</button>
    </div>
  </header>

  {#if !tauri}
    <div class="banner">This is the desk shell. Run <span class="mono">make dev</span> to talk to the local store. <span class="mono">make demo</span> proves the privacy invariants without the UI.</div>
  {/if}
  {#if download}
    <div class="dl-dock" class:work={download.active} class:bad={download.failed} role="status">
      <div class="dl-copy">
        <strong>
          {#if download.failed}
            Download failed
          {:else if download.active}
            {download.percent}%
          {:else}
            Ready
          {/if}
        </strong>
        <span>{download.message}</span>
      </div>
      <div class="dl-track" class:pulse={download.active && download.percent < 2}>
        <span style="width: {Math.max(download.active ? 4 : 0, download.percent)}%"></span>
      </div>
    </div>
  {:else if busy}
    <div class="banner">{busy}</div>
  {/if}
  {#if err}
    <div class="banner bad">{err}</div>
  {/if}
  {#if recoveries.length}
    <div class="banner recover" role="status">
      <p>A recording did not finish. Recover encrypts it on this Mac. Discard deletes only those chunks.</p>
      {#each recoveries as rec}
        <div class="recover-row">
          <strong>{rec.title}</strong>
          <span>{rec.chunk_count} chunks · {formatClock(rec.duration_ms)}</span>
          <button class="primary" disabled={!tauri || !!busy} onclick={() => recoverCapture(rec.session_id).catch(fail)}>Recover</button>
          <button class="ghost" disabled={!tauri || !!busy} onclick={() => (discardId = rec.session_id)}>Discard</button>
        </div>
      {/each}
    </div>
  {/if}

  <div class="body">
    <aside>
      <div class="aside-h">Sessions</div>
      <form
        class="filters"
        onsubmit={(e) => {
          e.preventDefault();
          runSearch().catch(fail);
        }}
      >
        <input bind:value={titleFilter} placeholder="Title contains" />
        <input type="date" bind:value={fromDay} aria-label="From date" />
        <input type="date" bind:value={toDay} aria-label="To date" />
        <input bind:value={tagFilter} placeholder="Tag" />
        <button type="submit" class="ghost">Filter</button>
      </form>
      {#if hits.length}
        <div class="hits">
          {#each hits as hit}
            <button class="row" onclick={() => openSession(hit.session_id).catch(fail)}>
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
          onclick={() => openSession(s.id).catch(fail)}
        >
          <strong>{s.title}</strong>
          <em>{s.status} · {s.id}</em>
        </button>
      {:else}
        <p class="empty">No sessions yet. Record on this Mac after consent. Transcribe with Apple on-device Speech, a local Whisper file, or Parakeet.</p>
      {/each}
    </aside>

    <main>
      {#if selected}
        <div class="title-row">
          <input class="title" bind:value={titleDraft} onchange={() => saveTitle().catch(fail)} />
          <button class="ghost" onclick={() => exportMd().catch(fail)}>Copy markdown</button>
          <button class="ghost" onclick={() => saveMd().catch(fail)}>Save markdown</button>
          <button class="ghost" onclick={() => removeSelected().catch(fail)}>Delete</button>
        </div>
        <div class="meta mono">
          {selected.session.status}
          {#if selected.audio_encrypted}· encrypted{/if}
          {#if selected.session.model_id}· {selected.session.model_id}{/if}
          {#if selected.tags.length}· {selected.tags.join(', ')}{/if}
        </div>
        <div class="title-row tags">
          <input class="tag-input" bind:value={tagDraft} placeholder="tags, comma separated" />
          <button class="ghost" onclick={() => saveTags().catch(fail)}>Save tags</button>
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
        {#if selected.key_points}
          <section>
            <h2>Key points</h2>
            <pre>{selected.key_points}</pre>
          </section>
        {/if}
        <section>
          <h2>Transcript</h2>
          {#if selected.segments.length}
            {#each selected.segments as seg}
              <p class="seg"><span class="mono t">{formatStamp(seg.start_ms)}</span> {seg.text}</p>
            {/each}
          {:else}
            <p class="empty">No transcript yet. Stop uses Apple on-device Speech when it is ready. You can also download Parakeet.</p>
            <div class="title-row">
              <button class="primary" disabled={!tauri || !!busy} onclick={() => transcribeSelected().catch(fail)}>
                Transcribe with {liveEngineLabel}
              </button>
              {#if parakeetEngine && !parakeetEngine.live_ready}
                <button class="ghost" disabled={!tauri || !!busy} onclick={() => downloadParakeet('int8').catch(fail)}>
                  Download Parakeet INT8
                </button>
              {/if}
            </div>
          {/if}
        </section>
      {:else}
        <div class="blank">
          <p class="wordmark">Quiet notes. Local audio.</p>
          <p>Sotto does not join the call. Live recordings stay encrypted here. Pick Apple Speech in the header — no download. Or download Parakeet INT8 from Models.</p>
          {#if parakeetEngine && !parakeetEngine.live_ready}
            <div class="title-row">
              <button class="ghost" disabled={!tauri || !!busy} onclick={() => downloadParakeet('int8').catch(fail)}>
                Download Parakeet INT8 (~670 MB)
              </button>
              <button class="ghost" onclick={() => openSettings().catch(fail)}>All engines</button>
            </div>
          {/if}
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
        <li>Live transcription uses Apple on-device Speech, or local Whisper / Parakeet weights. Fixture replay is only for make demo.</li>
      </ul>
      <p class="fine">You are responsible for telling others when the law requires consent to record.</p>
      <button class="primary" onclick={() => finishOnboarding().catch(fail)}>Enter the desk</button>
    </div>
  </div>
{/if}

{#if consentOpen}
  <div class="modal">
    <div class="card">
      <p class="wordmark">Before you record</p>
      <p class="fine">{sourceHint}</p>
      <blockquote>{consentText}</blockquote>
      <div class="actions">
        <button class="ghost" onclick={() => (consentOpen = false)}>Cancel</button>
        <button class="primary" onclick={() => acceptConsent().catch(fail)}>I can record</button>
      </div>
    </div>
  </div>
{/if}

{#if meetingAskOpen}
  <div class="modal">
    <div class="card">
      <p class="wordmark">A meeting looks open</p>
      <p>{meetingCopy}</p>
      <div class="actions">
        <button class="ghost" onclick={dismissMeetingAsk}>Not now</button>
        <button class="primary" onclick={() => acceptMeetingAsk().catch(fail)}>Record</button>
      </div>
    </div>
  </div>
{/if}

{#if deleteAllOpen}
  <div class="modal">
    <div class="card">
      <p class="wordmark">Delete everything on this Mac?</p>
      <p>Sessions, transcripts, and encrypted audio. This cannot be undone.</p>
      <div class="actions">
        <button class="ghost" onclick={() => (deleteAllOpen = false)}>Cancel</button>
        <button class="danger" onclick={() => confirmDeleteAll().catch(fail)}>Delete all</button>
      </div>
    </div>
  </div>
{/if}

{#if discardId}
  <div class="modal">
    <div class="card">
      <p class="wordmark">Discard this unfinished recording?</p>
      <p>Only the leftover chunks for that session are deleted. This cannot be undone.</p>
      <div class="actions">
        <button class="ghost" onclick={() => (discardId = null)}>Cancel</button>
        <button class="danger" onclick={() => confirmDiscardLive().catch(fail)}>Discard</button>
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
    <div
      class="card wide"
      role="dialog"
      aria-modal="true"
      aria-label="Models"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="sheet-head">
        <p class="wordmark">Models</p>
        <button class="solid" onclick={() => (settingsOpen = false)}>Done</button>
      </div>
      <p class="lead">Apple Speech needs no download. Parakeet and Whisper need weights you install. Fixture replay is demo-only.</p>
      {#if listedEngines.length}
        {#each listedEngines as engine}
          <div class="engine" class:is-default={defaultModel === engine.id}>
            <div class="engine-head">
              <strong>{engine.name}</strong>
              <span class="pill {engineTone(engine)}">{engineStatus(engine)}</span>
              {#if defaultModel === engine.id}
                <span class="pill current">Default</span>
              {/if}
            </div>
            <p class="engine-blurb">{engineBlurb(engine)}</p>
            {#if engine.id === 'parakeet-tdt-0.6b-v3'}
              <p class="engine-meta mono">decoder {parakeetStatus}</p>
              {#if download}
                <div class="dl-card" class:bad={download.failed} class:on={download.active}>
                  <p class="engine-meta">{download.message}</p>
                  <div class="dl-track" class:pulse={download.active && download.percent < 2}>
                    <span style="width: {Math.max(download.active ? 4 : 0, download.percent)}%"></span>
                  </div>
                  <p class="engine-meta mono">{download.percent}%</p>
                </div>
              {/if}
            {/if}
            <div class="engine-actions">
              {#if engine.id === 'apple-speech-ondevice'}
                <button
                  class={engine.live_ready ? 'solid' : 'ghost'}
                  onclick={() => setDefaultModel(engine.id).catch(fail)}
                >
                  {defaultModel === engine.id ? 'Using for live' : 'Use for live recordings'}
                </button>
              {:else if engine.id === 'parakeet-tdt-0.6b-v3'}
                <button class="solid" disabled={!!busy} onclick={() => downloadParakeet('int8').catch(fail)}>
                  {#if download?.active && download.variant === 'int8'}
                    Downloading INT8 {download.percent}%
                  {:else}
                    Download INT8 (~670 MB)
                  {/if}
                </button>
                <button class="ghost" onclick={() => importEngine(engine.id).catch(fail)}>Import folder</button>
                <button class="ghost" disabled={!!busy} onclick={() => downloadParakeet('fp32').catch(fail)}>
                  {#if download?.active && download.variant === 'fp32'}
                    Downloading FP32 {download.percent}%
                  {:else}
                    Download FP32 (~2.5 GB)
                  {/if}
                </button>
                <button class="ghost" onclick={() => setDefaultModel(engine.id).catch(fail)}>
                  {defaultModel === engine.id ? 'Using for live' : 'Use as default'}
                </button>
              {:else if engine.id === 'whisper-large-v3-turbo'}
                <button class="solid" onclick={() => importEngine(engine.id).catch(fail)}>Import ggml file</button>
                <button class="ghost" onclick={() => setDefaultModel(engine.id).catch(fail)}>
                  {defaultModel === engine.id ? 'Using for live' : 'Use as default'}
                </button>
              {:else}
                <button class="ghost" onclick={() => setDefaultModel(engine.id).catch(fail)}>
                  {defaultModel === engine.id ? 'Demo default' : 'Use only for demo'}
                </button>
              {/if}
              {#if engine.id !== 'fixture-replay' && engine.id !== 'apple-speech-ondevice' && (engine.live_ready || engine.install_state === 'ready')}
                <button class="ghost" onclick={() => removeEngine(engine.id).catch(fail)}>Remove weights</button>
              {/if}
            </div>
          </div>
        {/each}
      {:else}
        <p class="fine">Engine catalog loads when the desk talks to the local store.</p>
      {/if}
      <details class="more">
        <summary>Privacy, shortcuts, and this Mac</summary>
      <p class="wordmark privacy-h">Privacy</p>
      <div class="engine">
        <strong>Telemetry</strong>
        <span class="mono">{privacy.telemetry}</span>
        <p>Off unless you turn it on. Demo stays off.</p>
        <button
          class="ghost"
          onclick={() =>
            setPrivacy('telemetry', privacy.telemetry === 'on' ? 'off' : 'on').catch(fail)}
        >
          {privacy.telemetry === 'on' ? 'Turn off' : 'Turn on'}
        </button>
      </div>
      <div class="engine">
        <strong>Cloud mode</strong>
        <span class="mono">{privacy.cloud_mode}</span>
        <p>Cloud STT stays off unless you enable it. Fallback never selects cloud silently.</p>
        <button
          class="ghost"
          onclick={() =>
            setPrivacy('cloud_mode', privacy.cloud_mode === 'on' ? 'off' : 'on').catch(fail)}
        >
          {privacy.cloud_mode === 'on' ? 'Turn off' : 'Turn on'}
        </button>
      </div>
      <div class="engine">
        <strong>Retention</strong>
        <span class="mono">{privacy.retention_days} days</span>
        <p>0 keeps everything. A positive number deletes older sessions on this Mac.</p>
        <input
          class="tag-input"
          type="number"
          min="0"
          bind:value={privacy.retention_days}
          onchange={() =>
            api
              .settingsSet('retention_days', String(privacy.retention_days || '0'))
              .then(() => api.applyRetention())
              .then(() => api.privacy())
              .then((p) => (privacy = p))
              .catch(fail)}
        />
      </div>
      <div class="engine">
        <strong>Open at login</strong>
        <span class="mono">{login.backend} · {login.requested ? 'on' : 'off'}</span>
        <p>Registers Sotto as a login item on this Mac. It still will not record until you consent.</p>
        <button
          class="ghost"
          onclick={() =>
            api
              .loginSet(!login.requested)
              .then((r) => (login = r))
              .catch(fail)}
        >
          {login.requested ? 'Turn off' : 'Turn on'}
        </button>
      </div>
      <div class="engine">
        <strong>Record shortcut</strong>
        <span class="mono">{hotkeyMode}</span>
        <p>Fn is always armed on this Mac: tap to start or stop, hold to talk. It still cannot skip the consent card. macOS may ask for Input Monitoring. Command+Shift+Space remains a backup.</p>
        <div class="engine-actions">
          <input
            class="tag-input"
            bind:value={hotkeyShortcut}
            aria-label="Global shortcut"
            placeholder="CommandOrControl+Shift+Space"
          />
          <select
            class="tag-input"
            bind:value={hotkeyMode}
            aria-label="Shortcut mode"
          >
            <option value="toggle">Toggle</option>
            <option value="ptt">Push to talk</option>
          </select>
          <button class="ghost" onclick={() => saveHotkey().catch(fail)}>Save shortcut</button>
        </div>
      </div>
      <div class="engine">
        <strong>Ask when a meeting app is open</strong>
        <span class="mono">{meetingDetect ? 'on' : 'off'}</span>
        <p>Watches Zoom, Teams, and Slack on this Mac. It asks. It never starts capture without the consent card. Off by default. No calendar.</p>
        <button
          class="ghost"
          onclick={() =>
            api
              .meetingSet(!meetingDetect)
              .then((r) => {
                meetingDetect = r.enabled;
              })
              .catch(fail)}
        >
          {meetingDetect ? 'Turn off' : 'Turn on'}
        </button>
      </div>
      <div class="engine">
        <strong>System audio</strong>
        <span class="mono">{tapStatus}</span>
        <p>What you hear on this Mac via ScreenCaptureKit. Grant Screen Recording in System Settings if this says needs-permission. Mixed capture needs that plus the microphone and will not fall back to mic-only. Pick the source next to Record. Consent is still required. Tests never prompt.</p>
      </div>
      <div class="engine">
        <strong>Delete all</strong>
        <p>Removes sessions, search index, and encrypted audio on this Mac.</p>
        <button class="ghost" onclick={() => (deleteAllOpen = true)}>Delete all data</button>
      </div>
      </details>
    </div>
  </div>
{/if}

<style>
  .desk { height: 100vh; display: flex; flex-direction: column; background:
    radial-gradient(1200px 500px at 80% -10%, #2a2018 0%, transparent 50%),
    var(--shell); }
  header { display: flex; flex-wrap: wrap; gap: 12px 16px; align-items: center; padding: 14px 20px; border-bottom: 1px solid var(--line); }
  .brand { display: flex; gap: 8px; align-items: baseline; flex: 0 0 auto; }
  .wordmark { font-size: 28px; }
  header .wordmark { font-size: 26px; }
  .chip { font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase; background: var(--chip); padding: 4px 8px; color: var(--ok); }
  .chip.ghost { color: var(--mute); background: transparent; border: 1px solid var(--line); }
  .search { flex: 1 1 240px; min-width: 0; }
  .search input { width: 100%; background: var(--panel); border: 1px solid var(--line); color: var(--paper); padding: 10px 12px; }
  .rec { display: flex; gap: 8px; align-items: center; flex: 0 1 auto; flex-wrap: wrap; }
  .source-pick { background: transparent; border: 1px solid var(--line); color: var(--paper); padding: 8px 10px; }
  .led { width: 10px; height: 10px; border-radius: 50%; background: #3a2a26; box-shadow: inset 0 0 0 1px #000; }
  .led.on { background: var(--led); box-shadow: 0 0 12px var(--led); animation: pulse 1.2s ease-in-out infinite; }
  @keyframes pulse { 50% { opacity: 0.55; } }
  .clock { font-size: 18px; min-width: 4.5ch; }
  button { border: 0; background: var(--chip); color: var(--paper); padding: 8px 12px; }
  button.primary { background: var(--paper); color: var(--ink); font-weight: 600; }
  button.danger { background: var(--led); color: white; font-weight: 600; }
  button.ghost { background: transparent; border: 1px solid var(--line); }
  button:disabled { opacity: 0.4; cursor: not-allowed; }
  .banner { padding: 10px 20px; background: #241c14; color: var(--mute); font-size: 13px; }
  .banner.bad { background: #3a1814; color: #f3c0b6; }
  .banner.recover { background: #241810; color: var(--paper); display: grid; gap: 10px; }
  .recover-row { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .dl-dock {
    position: fixed;
    left: 16px;
    right: 16px;
    bottom: 16px;
    z-index: 80;
    padding: 14px 16px;
    background: var(--paper);
    color: var(--ink);
    border-radius: 12px;
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.45);
    font-weight: 600;
  }
  .dl-dock.bad { background: #3a1814; color: #f3c0b6; }
  .dl-copy { display: flex; gap: 10px; align-items: baseline; flex-wrap: wrap; }
  .dl-track { height: 8px; margin-top: 10px; background: #d9ccb4; border-radius: 99px; overflow: hidden; }
  .dl-dock.bad .dl-track { background: #5a241c; }
  .dl-track span {
    display: block; height: 100%; width: 0;
    background: var(--ink); border-radius: 99px;
    transition: width 180ms linear;
  }
  .dl-dock.bad .dl-track span { background: #f3c0b6; }
  .dl-track.pulse span { animation: pulse 1.2s ease-in-out infinite; width: 18% !important; }
  .dl-card { margin-top: 10px; padding: 10px 12px; background: #efe6d4; border: 1px solid #d9ccb4; }
  .dl-card.bad { background: #f3c0b6; border-color: #d23a22; }
  .dl-card.on { background: #1c1914; color: #f3ead8; }
  .dl-card.on .engine-meta { color: #d8ccb8; }
  .dl-card .dl-track { background: #3a322c; }
  .dl-card.on .dl-track span, .dl-card .dl-track span { background: var(--ok); }
  .body { flex: 1; display: grid; grid-template-columns: minmax(220px, 280px) 1fr; min-height: 0; }
  aside { border-right: 1px solid var(--line); overflow: auto; padding: 12px; }
  .aside-h { font-size: 11px; letter-spacing: 0.12em; text-transform: uppercase; color: var(--mute); margin-bottom: 8px; }
  .filters { display: grid; gap: 6px; margin-bottom: 12px; }
  .filters input { width: 100%; background: var(--panel); border: 1px solid var(--line); color: var(--paper); padding: 6px 8px; font-size: 12px; }
  .tag-input { flex: 1; background: var(--panel); border: 1px solid var(--line); color: var(--paper); padding: 6px 8px; }
  .title-row.tags { margin-bottom: 16px; }
  .row { display: flex; flex-direction: column; gap: 4px; width: 100%; text-align: left; background: transparent; padding: 10px; border: 1px solid transparent; color: var(--paper); margin-bottom: 4px; }
  .row em { color: var(--mute); font-style: normal; font-size: 12px; font-family: 'IBM Plex Mono', monospace; }
  .row.active, .row:hover { background: var(--panel); border-color: var(--line); }
  main { overflow: auto; padding: 28px 36px 64px; }
  .title-row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
  .title { flex: 1; min-width: 12rem; background: transparent; border: 0; border-bottom: 1px solid var(--line); color: var(--paper); font-family: 'Fraunces', serif; font-size: 32px; padding: 4px 0; }
  .meta { color: var(--mute); margin: 8px 0 24px; font-size: 12px; }
  h2 { font-size: 11px; letter-spacing: 0.14em; text-transform: uppercase; color: var(--mute); font-weight: 600; }
  .seg { line-height: 1.55; margin: 0 0 10px; max-width: 62ch; }
  .t { color: var(--mute); margin-right: 10px; font-size: 12px; }
  .empty, .fine { color: var(--mute); }
  .blank { max-width: 36rem; padding-top: 12vh; }
  .blank .wordmark { font-size: 42px; display: block; margin-bottom: 12px; }
  .modal { position: fixed; inset: 0; z-index: 20; background: rgba(10,8,6,0.72); display: grid; place-items: center; padding: 24px; }
  .card { background: var(--paper); color: var(--ink); padding: 28px; width: min(520px, 100%); max-height: 90vh; overflow: auto; }
  .card.wide { width: min(680px, 100%); }
  .card .wordmark { font-size: 32px; display: block; margin-bottom: 12px; }
  .privacy-h { font-size: 22px !important; margin-top: 16px; }
  .card ul { padding-left: 1.1rem; }
  .card blockquote { background: #e7dcc6; padding: 12px 14px; margin: 0 0 16px; }
  .card .fine { color: #6d6458; }
  .card button { color: var(--ink); background: #efe4cf; border: 1px solid #8f826c; }
  .card button.ghost { background: #fffaf0; color: var(--ink); border: 1px solid #8f826c; }
  .card button.solid, .card button.primary { background: var(--ink); color: var(--paper); border-color: var(--ink); font-weight: 600; }
  .card button.danger { background: var(--led); color: white; border-color: var(--led); }
  .card .tag-input { color: var(--ink); background: #fffaf0; border: 1px solid #8f826c; }
  .sheet-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 4px; }
  .sheet-head .wordmark { margin-bottom: 0; }
  .lead { color: #4f473c; margin: 0 0 8px; max-width: 52ch; }
  .engine { border-top: 1px solid #d9ccb4; padding: 14px 0; }
  .engine.is-default { background: #efe6d4; margin: 0 -12px; padding: 14px 12px; }
  .engine-head { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  .engine-blurb { margin: 6px 0 0; color: #4f473c; max-width: 52ch; }
  .engine-meta { margin: 4px 0 0; color: #6d6458; font-size: 12px; }
  .pill { font-size: 11px; letter-spacing: 0.04em; text-transform: uppercase; padding: 3px 8px; border: 1px solid #c4b79a; color: #4f473c; }
  .pill.ok { background: #d7e2c8; border-color: #8aa075; color: #243018; }
  .pill.wait { background: #f3e0b5; border-color: #c9a227; color: #3f3208; }
  .pill.demo { background: #ece4d4; color: #6d6458; }
  .pill.current { background: var(--ink); color: var(--paper); border-color: var(--ink); }
  .engine-actions { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; margin-top: 10px; }
  .more { margin-top: 8px; border-top: 1px solid #d9ccb4; padding-top: 8px; }
  .more summary { cursor: pointer; font-weight: 600; padding: 8px 0; }
  .actions { display: flex; gap: 8px; justify-content: flex-end; }
  pre { white-space: pre-wrap; font-family: inherit; }
  @media (max-width: 900px) {
    .body { grid-template-columns: 1fr; }
    aside { border-right: 0; border-bottom: 1px solid var(--line); max-height: 40vh; }
    header .wordmark { font-size: 22px; }
  }
</style>
