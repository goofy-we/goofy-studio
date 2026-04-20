import type { ImageModelDefinition } from '../../types';

export const COMFLY_GEMINI_FLASH_MODEL_ID = 'comfly/gemini-3.1-flash-image';

export const imageModel: ImageModelDefinition = {
  id: COMFLY_GEMINI_FLASH_MODEL_ID,
  mediaType: 'image',
  displayName: 'Gemini 3.1 Flash Image',
  providerId: 'comfly',
  description: 'Gemini 3.1 Flash 图像生成与编辑',
  eta: '1min',
  expectedDurationMs: 60000,
  defaultAspectRatio: '16:9',
  defaultResolution: '2K',
  aspectRatios: [
    { value: '1:1', label: '1:1' },
    { value: '1:4', label: '1:4' },
    { value: '1:8', label: '1:8' },
    { value: '9:16', label: '9:16' },
    { value: '16:9', label: '16:9' },
    { value: '3:4', label: '3:4' },
    { value: '4:3', label: '4:3' },
    { value: '4:1', label: '4:1' },
    { value: '8:1', label: '8:1' },
    { value: '2:3', label: '2:3' },
    { value: '3:2', label: '3:2' },
    { value: '5:4', label: '5:4' },
    { value: '4:5', label: '4:5' },
    { value: '21:9', label: '21:9' },
  ],
  resolutions: [
    { value: '0.5K', label: '0.5K' },
    { value: '1K', label: '1K' },
    { value: '2K', label: '2K' },
    { value: '4K', label: '4K' },
  ],
  resolveRequest: ({ referenceImageCount }) => ({
    requestModel: 'gemini-3.1-flash-image-preview',
    modeLabel: referenceImageCount > 0 ? '编辑模式' : '生成模式',
  }),
};