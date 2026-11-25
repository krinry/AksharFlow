export type SoundProfile = "off" | "blue" | "brown" | "red" | "topre";
export type SoundCue = "none" | "tap" | "error" | "space" | "backspace" | "line_complete";

class MechanicalSoundEngine {
  private ctx: AudioContext | null = null;
  private noiseBuffer: AudioBuffer | null = null;

  private ensureContext(): AudioContext | null {
    if (typeof window === "undefined") return null;
    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
    if (!AudioCtx) return null;

    if (!this.ctx) {
      this.ctx = new AudioCtx();
    }
    if (this.ctx.state === "suspended") {
      void this.ctx.resume();
    }
    if (!this.noiseBuffer) {
      const sampleRate = this.ctx.sampleRate;
      const length = Math.floor(sampleRate * 0.045);
      const buffer = this.ctx.createBuffer(1, length, sampleRate);
      const data = buffer.getChannelData(0);
      for (let i = 0; i < length; i++) {
        data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (sampleRate * 0.008));
      }
      this.noiseBuffer = buffer;
    }
    return this.ctx;
  }

  public play(cue: SoundCue, profile: SoundProfile, volume = 0.5): void {
    if (profile === "off" || cue === "none" || volume <= 0) return;
    const ctx = this.ensureContext();
    if (!ctx || !this.noiseBuffer) return;

    const now = ctx.currentTime;
    const gainLevel = Math.min(1, Math.max(0.02, volume));

    if (cue === "error") {
      this.playTone(ctx, now, 135, 95, 0.065, "triangle", 0.25 * gainLevel);
      return;
    }
    if (cue === "line_complete") {
      this.playTone(ctx, now, 523.25, 659.25, 0.09, "sine", 0.16 * gainLevel);
      this.playTone(ctx, now + 0.07, 659.25, 783.99, 0.12, "sine", 0.16 * gainLevel);
      return;
    }

    const isSpace = cue === "space";
    const isBackspace = cue === "backspace";
    const pitchJitter = 0.94 + Math.random() * 0.12;

    switch (profile) {
      case "blue": {
        this.playNoiseClick(
          ctx,
          now,
          (isSpace ? 2100 : 3400) * pitchJitter,
          2.6,
          0.026,
          0.42 * gainLevel,
        );
        this.playTone(
          ctx,
          now,
          (isSpace ? 170 : isBackspace ? 220 : 280) * pitchJitter,
          110,
          0.024,
          "square",
          0.09 * gainLevel,
        );
        break;
      }
      case "brown": {
        this.playNoiseClick(
          ctx,
          now,
          (isSpace ? 1500 : 2200) * pitchJitter,
          1.6,
          0.024,
          0.32 * gainLevel,
        );
        this.playTone(
          ctx,
          now,
          (isSpace ? 140 : 190) * pitchJitter,
          95,
          0.028,
          "triangle",
          0.14 * gainLevel,
        );
        break;
      }
      case "red": {
        this.playNoiseClick(
          ctx,
          now,
          (isSpace ? 1100 : 1650) * pitchJitter,
          1.1,
          0.02,
          0.24 * gainLevel,
        );
        this.playTone(
          ctx,
          now,
          (isSpace ? 115 : 145) * pitchJitter,
          70,
          0.025,
          "sine",
          0.16 * gainLevel,
        );
        break;
      }
      case "topre": {
        this.playNoiseClick(
          ctx,
          now,
          (isSpace ? 780 : 1050) * pitchJitter,
          0.9,
          0.032,
          0.28 * gainLevel,
        );
        this.playTone(
          ctx,
          now,
          (isSpace ? 105 : 135) * pitchJitter,
          58,
          0.04,
          "sine",
          0.26 * gainLevel,
        );
        break;
      }
    }
  }

  private playNoiseClick(
    ctx: AudioContext,
    startTime: number,
    freq: number,
    q: number,
    duration: number,
    volume: number,
  ): void {
    if (!this.noiseBuffer) return;
    const src = ctx.createBufferSource();
    src.buffer = this.noiseBuffer;

    const filter = ctx.createBiquadFilter();
    filter.type = "bandpass";
    filter.frequency.setValueAtTime(freq, startTime);
    filter.Q.setValueAtTime(q, startTime);

    const gain = ctx.createGain();
    gain.gain.setValueAtTime(volume, startTime);
    gain.gain.exponentialRampToValueAtTime(0.001, startTime + duration);

    src.connect(filter);
    filter.connect(gain);
    gain.connect(ctx.destination);
    src.start(startTime);
    src.stop(startTime + duration);
  }

  private playTone(
    ctx: AudioContext,
    startTime: number,
    freqStart: number,
    freqEnd: number,
    duration: number,
    type: OscillatorType,
    volume: number,
  ): void {
    const osc = ctx.createOscillator();
    osc.type = type;
    osc.frequency.setValueAtTime(freqStart, startTime);
    osc.frequency.exponentialRampToValueAtTime(Math.max(20, freqEnd), startTime + duration);

    const gain = ctx.createGain();
    gain.gain.setValueAtTime(volume, startTime);
    gain.gain.exponentialRampToValueAtTime(0.001, startTime + duration);

    osc.connect(gain);
    gain.connect(ctx.destination);
    osc.start(startTime);
    osc.stop(startTime + duration);
  }
}

export const soundEngine = new MechanicalSoundEngine();
