export type Session = {
  id: string;
  created_at: string;
  started_at: string | null;
  ended_at: string | null;
  title: string;
  status: string;
  model_id: string | null;
  language: string | null;
  duration_seconds: number | null;
  consent_state: string;
  notes: string | null;
  source: string;
};

export type Segment = {
  start_ms: number;
  end_ms: number;
  text: string;
  confidence: number | null;
};

export type SessionDetail = {
  session: Session;
  transcript: string | null;
  summary: string | null;
  action_items: string | null;
  key_points: string | null;
  segments: Segment[];
  audio_encrypted: boolean;
  audio_path: string | null;
};

export type SearchHit = {
  session_id: string;
  title: string;
  snippet: string;
};

export type Engine = {
  id: string;
  vendor: string;
  name: string;
  version: string;
  mode: string;
  estimated_speed: string;
  estimated_accuracy: string;
  install_state: string;
  disk_size_mb: number;
  notes: string;
  supported_languages: string[];
};
