# Desk closeout — Design

Usability of APIs that already exist. Do not implement live Core Audio taps here (that is `live-record`).

## Invoke guard

`src/lib/api.ts` `call()` returns a `DeskError` when not in Tauri. Every desk action uses `call()`. No raw `invoke` from routes.

## Settings modal

Overlay click closes. Card click does not close (`stopPropagation`). Engine catalog from `engines_list`. Default model is `settings.default_model`. Install uses the existing local-file command + save/open dialog. Delete uses `model_delete`. Fixture-replay is not deletable.

## Search filters

A Filter control (and Enter in filter fields) runs `search_filtered` with title / date / tag even when the top query is empty.

## Busy + delete-all

Stop → show “Transcribing…” until `transcribe_run` returns. Settings includes delete-all with a confirm.

## Layout

Header wraps. Search and record controls must remain usable around 1024px. No purple gradients. LED + on this Mac stay visible.
