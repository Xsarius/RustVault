/**
 * Vitest global test setup.
 */

import "@testing-library/jest-dom";

// ---------------------------------------------------------------------------
// jsdom polyfills
// ---------------------------------------------------------------------------

// jsdom does not ship a Touch constructor, but it does dispatch TouchEvents.
// Provide a minimal stub so touch-gesture tests can construct Touch objects.
if (!globalThis.Touch) {
  class TouchPolyfill {
    readonly identifier: number;
    readonly target: EventTarget;
    readonly clientX: number;
    readonly clientY: number;
    readonly pageX: number;
    readonly pageY: number;
    readonly screenX: number;
    readonly screenY: number;
    readonly force: number;
    readonly radiusX: number;
    readonly radiusY: number;
    readonly rotationAngle: number;

    constructor(init: TouchInit) {
      this.identifier = init.identifier;
      this.target = init.target;
      this.clientX = init.clientX ?? 0;
      this.clientY = init.clientY ?? 0;
      this.pageX = init.pageX ?? init.clientX ?? 0;
      this.pageY = init.pageY ?? init.clientY ?? 0;
      this.screenX = init.screenX ?? 0;
      this.screenY = init.screenY ?? 0;
      this.force = init.force ?? 1;
      this.radiusX = init.radiusX ?? 0;
      this.radiusY = init.radiusY ?? 0;
      this.rotationAngle = init.rotationAngle ?? 0;
    }
  }
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (globalThis as any).Touch = TouchPolyfill;
}
