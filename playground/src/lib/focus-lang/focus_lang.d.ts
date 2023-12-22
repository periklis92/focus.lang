/* tslint:disable */
/* eslint-disable */
/**
*/
export class ModuleLoader {
  free(): void;
/**
* @param {string} _root
* @returns {ModuleLoader}
*/
  static new(_root: string): ModuleLoader;
/**
* @param {string} ident
* @param {string} source
* @returns {number}
*/
  load_module_from_source(ident: string, source: string): number;
}
/**
*/
export class Vm {
  free(): void;
/**
* @param {string} type_
* @param {Function} _function
*/
  add_event_listener(type_: string, _function: Function): void;
/**
* @param {ModuleLoader} module_loader
* @returns {Vm}
*/
  static new(module_loader: ModuleLoader): Vm;
/**
* @returns {Vm}
*/
  static new_with_std(): Vm;
/**
* @param {string} source
*/
  execute_from_source(source: string): void;
/**
* @param {number} index
* @param {string} ident
*/
  execute_module(index: number, ident: string): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_vm_free: (a: number) => void;
  readonly vm_add_event_listener: (a: number, b: number, c: number, d: number) => void;
  readonly vm_new: (a: number) => number;
  readonly vm_new_with_std: () => number;
  readonly vm_execute_from_source: (a: number, b: number, c: number, d: number) => void;
  readonly vm_execute_module: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_moduleloader_free: (a: number) => void;
  readonly moduleloader_new: (a: number, b: number) => number;
  readonly moduleloader_load_module_from_source: (a: number, b: number, c: number, d: number, e: number) => number;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
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
