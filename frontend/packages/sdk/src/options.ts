import type { ApiOptions } from "@polkadot/api/types";

import { typeDefinitions } from '@polkadot/types';
import { types } from './lib/types';

export function options(options: ApiOptions = {}): ApiOptions {
  return {
    ...options,
    types: {
      ...types,
      ...typeDefinitions,
      ...options.types || {},
    } as unknown as ApiOptions['types'],
  }
}