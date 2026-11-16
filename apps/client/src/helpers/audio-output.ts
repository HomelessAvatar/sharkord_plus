import { toast } from 'sonner';

type TAudioElementWithSinkId = HTMLAudioElement & {
  setSinkId?: (deviceId: string) => Promise<void>;
  sinkId?: string;
};

const OUTPUT_WARNING_COOLDOWN_MS = 10000;
const lastOutputWarningByDevice = new Map<string, number>();

const applyAudioOutputDevice = async (
  audioElement: HTMLAudioElement | null,
  playbackId: string | undefined,
  showWarning = true
) => {
  if (!audioElement || !playbackId) return true;

  const audioElementWithSink = audioElement as TAudioElementWithSinkId;

  if (typeof audioElementWithSink.setSinkId !== 'function') {
    return true;
  }

  // In HTMLMediaElement W3C spec, sinkId === '' denotes the system default audio output device.
  // If the target is 'default' (or empty) and the element is already targeting default ('default' or ''),
  // no setSinkId invocation is needed and it avoids triggering browser permission rejections (e.g. in Firefox).
  const currentSink = audioElementWithSink.sinkId ?? '';
  const isTargetDefault = playbackId === 'default' || playbackId === '';
  const isCurrentDefault = currentSink === '' || currentSink === 'default';

  if (currentSink === playbackId || (isTargetDefault && isCurrentDefault)) {
    return true;
  }

  try {
    // W3C spec specifies passing empty string "" to route to the user agent's default output
    await audioElementWithSink.setSinkId(isTargetDefault ? '' : playbackId);

    return true;
  } catch (error) {
    console.warn('Failed to set output device:', error);

    const isNotAllowed =
      error instanceof DOMException &&
      (error.name === 'NotAllowedError' || error.name === 'SecurityError');

    const now = Date.now();
    const lastWarningAt = lastOutputWarningByDevice.get(playbackId) ?? 0;
    const isOutsideCooldown = now - lastWarningAt > OUTPUT_WARNING_COOLDOWN_MS;

    if (showWarning && isOutsideCooldown && !isTargetDefault && !isNotAllowed) {
      lastOutputWarningByDevice.set(playbackId, now);
      toast.warning(
        'Could not switch to the selected speaker. Using default output.'
      );
    }

    return false;
  }
};

export { applyAudioOutputDevice };
