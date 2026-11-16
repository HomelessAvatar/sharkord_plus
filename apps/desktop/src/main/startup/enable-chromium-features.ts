import electron from 'electron';

const LINUX_HARDWARE_VIDEO_CODECS = [
  'AcceleratedVideoEncoder',
  'AcceleratedVideoDecodeLinuxGL',
  'AcceleratedVideoDecodeLinuxZeroCopyGL',
  'VaapiIgnoreDriverChecks'
];

const WINDOWS_HARDWARE_VIDEO_CODECS = [
  'AcceleratedVideoEncoder',
  'MediaFoundationVideoEncodeAccelerator',
  'D3D11VideoDecoder',
  'PlatformHEVCDecoderSupport'
];

const DEFAULT_HARDWARE_VIDEO_CODECS = ['AcceleratedVideoEncoder'];

const getHardwareVideoFeatureList = (platform: NodeJS.Platform): string[] => {
  switch (platform) {
    case 'win32':
      return [...WINDOWS_HARDWARE_VIDEO_CODECS];
    case 'linux':
      return [...LINUX_HARDWARE_VIDEO_CODECS];
    default:
      return [...DEFAULT_HARDWARE_VIDEO_CODECS];
  }
};

const enableChromiumFeatures = () => {
  const features = getHardwareVideoFeatureList(process.platform);

  if (features.length === 0) {
    return;
  }

  // default import so that tests can load the file without an electron runtime
  electron.app.commandLine.appendSwitch('enable-features', features.join(','));
};

export { enableChromiumFeatures, getHardwareVideoFeatureList };
