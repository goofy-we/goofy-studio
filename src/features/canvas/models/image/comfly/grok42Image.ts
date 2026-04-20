import type { ImageModelDefinition } from '../../types';

export const COMFLY_GROK42_IMAGE_MODEL_ID = 'comfly/grok-4.2-image';

export const imageModel: ImageModelDefinition = {
  id: COMFLY_GROK42_IMAGE_MODEL_ID,
  mediaType: 'image',
  displayName: 'Grok 4.2 Image',
  providerId: 'comfly',
  description: 'Grok 4.2 高质量图像生成与编辑',
  eta: '30s',
  expectedDurationMs: 30000,
  defaultAspectRatio: '1:1',
  defaultResolution: '2K',
  aspectRatios: [
    { value: '1:1', label: '1:1' },
    { value: '2:3', label: '2:3' },
    { value: '3:2', label: '3:2' },
    { value: '9:16', label: '9:16' },
    { value: '16:9', label: '16:9' },
  ],
  resolutions: [
    { value: '1K', label: '1K' },
    { value: '2K', label: '2K' },
    { value: '4K', label: '4K' },
  ],
  resolveRequest: ({ referenceImageCount }) => ({
    requestModel: 'grok-4.2-image',
    modeLabel: referenceImageCount > 0 ? '编辑模式' : '生成模式',
  }),
};