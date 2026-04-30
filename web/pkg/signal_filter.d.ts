/* tslint:disable */
/* eslint-disable */

export class FilterEngine {
    free(): void;
    [Symbol.dispose](): void;
    compute_coefficients(sample_rate: number, cutoff: number, order: number, filter_type: FilterType): void;
    cutoff(): number;
    filter_type(): FilterType;
    constructor();
    order(): number;
    process_batch(data: Float64Array): Float64Array;
    reset_state(): void;
    sample_rate(): number;
}

export enum FilterType {
    LowPass = 0,
    HighPass = 1,
    BandPass = 2,
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_filterengine_free: (a: number, b: number) => void;
    readonly filterengine_compute_coefficients: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly filterengine_cutoff: (a: number) => number;
    readonly filterengine_filter_type: (a: number) => number;
    readonly filterengine_new: () => number;
    readonly filterengine_order: (a: number) => number;
    readonly filterengine_process_batch: (a: number, b: number) => number;
    readonly filterengine_reset_state: (a: number) => void;
    readonly filterengine_sample_rate: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
