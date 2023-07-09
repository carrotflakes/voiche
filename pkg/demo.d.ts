/* tslint:disable */
/* eslint-disable */
/**
*/
export class Processor {
  free(): void;
/**
*/
  constructor();
/**
* @param {Float32Array} buffer
*/
  process(buffer: Float32Array): void;
/**
* @param {number} pitch
*/
  set_pitch(pitch: number): void;
/**
* @param {number} formant
*/
  set_formant(formant: number): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_processor_free: (a: number) => void;
  readonly processor_new: () => number;
  readonly processor_process: (a: number, b: number, c: number, d: number) => void;
  readonly processor_set_pitch: (a: number, b: number) => void;
  readonly processor_set_formant: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
