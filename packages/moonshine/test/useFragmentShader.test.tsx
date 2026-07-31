import { test, expect, describe, beforeAll, afterAll } from "bun:test";
import { GlobalRegistrator } from "@happy-dom/global-registrator";
import { renderHook, act } from "@testing-library/react";
import { useFragmentShader } from "../src/shaders";
import * as React from "react";

beforeAll(() => {
  GlobalRegistrator.register();
});
afterAll(() => {
  GlobalRegistrator.unregister();
});

const getMockGLContext = (overrides = {}) => {
  return {
    VERTEX_SHADER: 35633,
    FRAGMENT_SHADER: 35632,
    createShader: () => ({}),
    shaderSource: () => {},
    compileShader: () => {},
    getShaderParameter: () => true,
    getShaderInfoLog: () => "",
    deleteShader: () => {},
    createProgram: () => ({}),
    attachShader: () => {},
    linkProgram: () => {},
    getProgramParameter: () => true,
    getProgramInfoLog: () => "",
    useProgram: () => {},
    getAttribLocation: () => 0,
    enableVertexAttribArray: () => {},
    vertexAttribPointer: () => {},
    createBuffer: () => ({}),
    bindBuffer: () => {},
    bufferData: () => {},
    clearColor: () => {},
    clear: () => {},
    getUniformLocation: () => null,
    uniform2f: () => {},
    uniform1f: () => {},
    drawArrays: () => {},
    deleteProgram: () => {},
    deleteBuffer: () => {},
    viewport: () => {},
    canvas: { width: 800, height: 600, clientWidth: 800, clientHeight: 600 },
    ...overrides
  } as unknown as WebGLRenderingContext;
};

describe("useFragmentShader", () => {
  test("initializes webgl context and handles basic drawing", () => {
    let createShaderCalled = 0;

    let rafCallback: any = null;
    window.requestAnimationFrame = (cb) => { rafCallback = cb; return 1; };
    window.cancelAnimationFrame = () => { rafCallback = null; };

    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, type: string, options?: any) {
      if (type === "webgl2" || type === "webgl") {
        return getMockGLContext({
          createShader: () => { createShaderCalled++; return {}; },
          canvas: this
        });
      }
      return origGetContext.apply(this, [type, options] as any);
    } as any;

    const { result, unmount } = renderHook(
      ({ source, opts }) => {
        const res = useFragmentShader(source, opts);
        if (!res.canvasRef.current) {
          const canvas = document.createElement("canvas");
          res.canvasRef.current = canvas;
        }
        return res;
      },
      { initialProps: { source: "void main() {}", opts: { animate: false } } }
    );

    expect(createShaderCalled).toBeGreaterThan(0);
    expect(result.current.gl).not.toBeNull();

    unmount();
    HTMLCanvasElement.prototype.getContext = origGetContext;
  });

  test("handles compile errors gracefully", () => {
    let consoleErrorArgs: any[] = [];
    const origError = console.error;
    console.error = (...args) => { consoleErrorArgs.push(args); };

    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, type: string, options?: any) {
      if (type === "webgl2" || type === "webgl") {
        return getMockGLContext({
          getShaderParameter: () => false,
          getShaderInfoLog: () => "mock compile error",
          canvas: this
        });
      }
      return origGetContext.apply(this, [type, options] as any);
    } as any;

    const { result, unmount } = renderHook(
      () => {
        const res = useFragmentShader("invalid shader");
        if (!res.canvasRef.current) {
          const canvas = document.createElement("canvas");
          res.canvasRef.current = canvas;
        }
        return res;
      }
    );

    expect(consoleErrorArgs.length).toBe(1);
    expect(consoleErrorArgs[0][0].message).toContain("mock compile error");
    expect(result.current.gl).toBeNull();

    unmount();
    HTMLCanvasElement.prototype.getContext = origGetContext;
    console.error = origError;
  });

  test("setSource allows dynamic shader updates", () => {
    let compileCount = 0;
    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, type: string, options?: any) {
      if (type === "webgl2" || type === "webgl") {
        return getMockGLContext({
          compileShader: () => { compileCount++; },
          canvas: this
        });
      }
      return origGetContext.apply(this, [type, options] as any);
    } as any;

    const { result, unmount } = renderHook(
      () => {
        const res = useFragmentShader("void main() {}", { animate: false });
        if (!res.canvasRef.current) {
          res.canvasRef.current = document.createElement("canvas");
        }
        return res;
      }
    );

    expect(compileCount).toBe(2);

    act(() => {
      result.current.setSource("void main() { color = vec4(1); }");
    });

    expect(compileCount).toBe(4);

    unmount();
    HTMLCanvasElement.prototype.getContext = origGetContext;
  });

  test("animation frame loop triggers onFrame callback", () => {
    let frameCount = 0;
    let rafCallback: any = null;
    window.requestAnimationFrame = (cb) => { rafCallback = cb; return 1; };
    window.cancelAnimationFrame = () => { rafCallback = null; };

    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, type: string, options?: any) {
      if (type === "webgl2" || type === "webgl") {
        return getMockGLContext({ canvas: this });
      }
      return origGetContext.apply(this, [type, options] as any);
    } as any;

    const { result, unmount } = renderHook(
      () => {
        const res = useFragmentShader("void main() {}", {
          animate: true,
          onFrame: () => { frameCount++; }
        });
        if (!res.canvasRef.current) {
          res.canvasRef.current = document.createElement("canvas");
        }
        return res;
      }
    );

    expect(frameCount).toBe(1); // initial tick
    expect(rafCallback).not.toBeNull();

    // Simulate animation frame
    act(() => {
      if (rafCallback) rafCallback(16);
    });

    expect(frameCount).toBe(2);

    unmount();

    // Check cleanup
    expect(rafCallback).toBeNull();
    HTMLCanvasElement.prototype.getContext = origGetContext;
  });

  test("throws when WebGL is unavailable", () => {
    let consoleErrorArgs: any[] = [];
    const origError = console.error;
    console.error = (...args) => { consoleErrorArgs.push(args); };

    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function() {
      return null;
    } as any;

    const { result, unmount } = renderHook(
      () => {
        const res = useFragmentShader("void main() {}");
        if (!res.canvasRef.current) {
          const canvas = document.createElement("canvas");
          res.canvasRef.current = canvas;
        }
        return res;
      }
    );

    expect(consoleErrorArgs.length).toBeGreaterThan(0);
    expect(consoleErrorArgs[0][0].message).toContain("WebGL unavailable");
    expect(result.current.gl).toBeNull();

    unmount();
    HTMLCanvasElement.prototype.getContext = origGetContext;
    console.error = origError;
  });

  test("does not animate if options.animate is false", () => {
    let frameCount = 0;
    let rafCalled = 0;
    const origRaf = window.requestAnimationFrame;
    window.requestAnimationFrame = (cb) => { rafCalled++; return 1; };

    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, type: string, options?: any) {
      if (type === "webgl2" || type === "webgl") {
        return getMockGLContext({ canvas: this });
      }
      return origGetContext.apply(this, [type, options] as any);
    } as any;

    const { result, unmount } = renderHook(
      () => {
        const res = useFragmentShader("void main() {}", {
          animate: false,
          onFrame: () => { frameCount++; }
        });
        if (!res.canvasRef.current) {
          res.canvasRef.current = document.createElement("canvas");
        }
        return res;
      }
    );

    // Initial tick happens synchronously in useEffect, so onFrame might be called once,
    // but requestAnimationFrame shouldn't be called to loop it.
    expect(frameCount).toBe(1);
    expect(rafCalled).toBe(0);

    unmount();
    HTMLCanvasElement.prototype.getContext = origGetContext;
    window.requestAnimationFrame = origRaf;
  });

  test("cleans up resources on unmount", () => {
    let deleteProgramCalled = 0;
    let deleteBufferCalled = 0;

    const origGetContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, type: string, options?: any) {
      if (type === "webgl2" || type === "webgl") {
        return getMockGLContext({
          createProgram: () => ({}),
          createBuffer: () => ({}),
          deleteProgram: () => { deleteProgramCalled++; },
          deleteBuffer: () => { deleteBufferCalled++; },
          canvas: this
        });
      }
      return origGetContext.apply(this, [type, options] as any);
    } as any;

    const { result, unmount } = renderHook(
      () => {
        const res = useFragmentShader("void main() {}");
        if (!res.canvasRef.current) {
          res.canvasRef.current = document.createElement("canvas");
        }
        return res;
      }
    );

    // Initial render sets up
    expect(result.current.gl).not.toBeNull();
    expect(deleteProgramCalled).toBe(0);
    expect(deleteBufferCalled).toBe(0);

    // Unmount should clean up
    unmount();

    expect(deleteProgramCalled).toBe(1);
    expect(deleteBufferCalled).toBe(0); // buffer is no longer created in current buildProgram logic for webgl2 default
    // Note: glRef is still accessible via the getter but we can check if bundle was destroyed

    HTMLCanvasElement.prototype.getContext = origGetContext;
  });
});
