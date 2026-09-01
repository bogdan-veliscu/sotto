//! Mix microphone and system-audio PCM into one stream.

/// Saturating average of two mono lanes. The shorter lane is padded with silence.
pub fn mix_pcm(mic: &[i16], system: &[i16]) -> Vec<i16> {
    let n = mic.len().max(system.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let a = i32::from(mic.get(i).copied().unwrap_or(0));
        let b = i32::from(system.get(i).copied().unwrap_or(0));
        out.push(((a + b) / 2).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16);
    }
    out
}
