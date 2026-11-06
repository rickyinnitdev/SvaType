import { preferences } from './stores/settings.svelte';
type Tone = 'key' | 'error' | 'success';
let context: AudioContext | undefined;
const buffers: Partial<Record<Tone, AudioBuffer>> = {};
let sources: AudioBufferSourceNode[] = [];
let epoch = 0;
/** Tiny original PCM cues, generated only on use. No audio files or idle work. */
export function sound(tone: Tone) {
  if (!preferences.sound || document.hidden || !document.hasFocus()) return;
  try {
    const audio = (context ??= new AudioContext());
    const generation = epoch;
    let buffer = buffers[tone];
    if (!buffer) {
      const [frequency, duration] =
        tone === 'error'
          ? [170, 0.065]
          : tone === 'success'
            ? [660, 0.09]
            : [440, 0.025];
      buffer = audio.createBuffer(1, Math.ceil(22050 * duration), 22050);
      const samples = buffer.getChannelData(0);
      for (let i = 0; i < samples.length; i++) {
        const phase = i / samples.length;
        const envelope = Math.min(1, phase * 12) * (1 - phase) ** 2;
        samples[i] =
          0.06 * envelope * Math.sin((2 * Math.PI * frequency * i) / 22050);
      }
      buffers[tone] = buffer;
    }
    const selected = buffer;
    void audio
      .resume()
      .then(() => {
        if (epoch !== generation || !preferences.sound || document.hidden) {
          if (!sources.length) void audio.suspend();
          return;
        }
        if (sources.length >= 8) return;
        const source = audio.createBufferSource();
        source.buffer = selected;
        source.connect(audio.destination);
        sources.push(source);
        source.onended = () => {
          source.disconnect();
          sources = sources.filter((item) => item !== source);
          if (!sources.length) void audio.suspend();
        };
        source.start();
      })
      .catch(() => {});
  } catch {
    /* Audio is optional; input remains usable when no output device exists. */
  }
}
export function pauseSounds() {
  epoch++;
  for (const source of sources) {
    try {
      source.stop();
    } catch {
      /* Already ended. */
    }
  }
  sources = [];
  if (context) void context.suspend().catch(() => {});
}
