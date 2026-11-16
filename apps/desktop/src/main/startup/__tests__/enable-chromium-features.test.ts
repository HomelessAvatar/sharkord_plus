import { describe, expect, mock, test } from 'bun:test';

const appendSwitchMock = mock(() => {});

mock.module('electron', () => ({
  default: {
    app: {
      commandLine: {
        appendSwitch: appendSwitchMock
      }
    }
  },
  app: {
    commandLine: {
      appendSwitch: appendSwitchMock
    }
  }
}));

const { enableChromiumFeatures, getHardwareVideoFeatureList } =
  await import('../enable-chromium-features');

describe('getHardwareVideoFeatureList', () => {
  test('returns Windows GPU encoding and HEVC decoding features for win32', () => {
    const features = getHardwareVideoFeatureList('win32');

    expect(features).toEqual([
      'AcceleratedVideoEncoder',
      'MediaFoundationVideoEncodeAccelerator',
      'D3D11VideoDecoder',
      'PlatformHEVCDecoderSupport'
    ]);
  });

  test('returns VA-API decode and encode features for linux', () => {
    const features = getHardwareVideoFeatureList('linux');

    expect(features).toEqual([
      'AcceleratedVideoEncoder',
      'AcceleratedVideoDecodeLinuxGL',
      'AcceleratedVideoDecodeLinuxZeroCopyGL',
      'VaapiIgnoreDriverChecks'
    ]);
  });

  test('returns default accelerated video encoder for darwin', () => {
    const features = getHardwareVideoFeatureList('darwin');

    expect(features).toEqual(['AcceleratedVideoEncoder']);
  });

  test('returns default accelerated video encoder for other platforms', () => {
    const features = getHardwareVideoFeatureList('freebsd');

    expect(features).toEqual(['AcceleratedVideoEncoder']);
  });
});

describe('enableChromiumFeatures', () => {
  test('appends enable-features switch with platform features to electron command line', () => {
    appendSwitchMock.mockClear();

    enableChromiumFeatures();

    const expectedFeatures = getHardwareVideoFeatureList(process.platform).join(
      ','
    );

    expect(appendSwitchMock).toHaveBeenCalledWith(
      'enable-features',
      expectedFeatures
    );
  });
});
