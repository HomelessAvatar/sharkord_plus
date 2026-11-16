import { describe, expect, test } from 'bun:test';
import { isHardwareEncoder } from '../stats-popover';

describe('isHardwareEncoder', () => {
  test('recognizes hardware encoders', () => {
    expect(
      isHardwareEncoder(
        'MediaFoundationVideoEncodeAccelerator (NVIDIA H.264 Encoder MFT)'
      )
    ).toBe(true);
    expect(isHardwareEncoder('D3D12VideoEncodeAccelerator')).toBe(true);
    expect(isHardwareEncoder('D3D11VideoDecoder')).toBe(true);
  });

  test('recognizes software encoders', () => {
    expect(isHardwareEncoder('OpenH264')).toBe(false);
    expect(isHardwareEncoder('libaom')).toBe(false);
    expect(isHardwareEncoder('libvpx')).toBe(false);
  });

  test('returns null for unknown encoders', () => {
    expect(isHardwareEncoder('UnknownCustomEncoder')).toBeNull();
  });
});
