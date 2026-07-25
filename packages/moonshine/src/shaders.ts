import { useEffect, useRef, type RefObject } from "react";

const DEFAULT_VERTEX = `#version 300 es
precision highp float;
const vec2 verts[3] = vec2[3](
  vec2(-1.0, -1.0),
  vec2( 3.0, -1.0),
  vec2(-1.0,  3.0)
);
void main() {
  gl_Position = vec4(verts[gl_VertexID], 0.0, 1.0);
}
`;

const DEFAULT_VERTEX_WEBGL1 = `
attribute vec2 a_pos;
void main() {
  gl_Position = vec4(a_pos, 0.0, 1.0);
}
`;

export type FragmentShaderHandle = {
  canvasRef: RefObject<HTMLCanvasElement | null>;
  /** WebGL context once mounted, else null. */
  gl: WebGLRenderingContext | WebGL2RenderingContext | null;
  /** Recompile with a new fragment source. */
  setSource: (source: string) => void;
};

export type UseFragmentShaderOptions = {
  /** Prefer WebGL2 (default true). Falls back to WebGL1. */
  webgl2?: boolean;
  /** Device pixel ratio multiplier (default devicePixelRatio). */
  dpr?: number;
  /** Uniform `u_time` in seconds from mount. */
  animate?: boolean;
  /** Called each frame after draw (when animate). */
  onFrame?: (t: number, gl: WebGLRenderingContext | WebGL2RenderingContext) => void;
};

function compileShader(
  gl: WebGLRenderingContext | WebGL2RenderingContext,
  type: number,
  source: string,
): WebGLShader {
  const shader = gl.createShader(type);
  if (!shader) throw new Error("useFragmentShader: createShader failed");
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
    const info = gl.getShaderInfoLog(shader) ?? "unknown compile error";
    gl.deleteShader(shader);
    throw new Error(`useFragmentShader: shader compile failed: ${info}`);
  }
  return shader;
}

function linkProgram(
  gl: WebGLRenderingContext | WebGL2RenderingContext,
  vert: WebGLShader,
  frag: WebGLShader,
): WebGLProgram {
  const program = gl.createProgram();
  if (!program) throw new Error("useFragmentShader: createProgram failed");
  gl.attachShader(program, vert);
  gl.attachShader(program, frag);
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    const info = gl.getProgramInfoLog(program) ?? "unknown link error";
    gl.deleteProgram(program);
    throw new Error(`useFragmentShader: program link failed: ${info}`);
  }
  return program;
}

/** Wrap a fragment body into a complete WebGL2 shader if needed. */
export function wrapFragmentSource(source: string, webgl2: boolean): string {
  const trimmed = source.trim();
  if (trimmed.startsWith("#version") || trimmed.includes("void main")) {
    return source;
  }
  if (webgl2) {
    return `#version 300 es
precision highp float;
uniform vec2 u_resolution;
uniform float u_time;
out vec4 outColor;
${source}
void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  outColor = shade(uv, u_time);
}
`;
  }
  return `
precision highp float;
uniform vec2 u_resolution;
uniform float u_time;
${source}
void main() {
  vec2 uv = gl_FragCoord.xy / u_resolution;
  gl_FragColor = shade(uv, u_time);
}
`;
}

type ProgramBundle = {
  gl: WebGLRenderingContext | WebGL2RenderingContext;
  program: WebGLProgram;
  isWebgl2: boolean;
  buffer: WebGLBuffer | null;
};

function buildProgram(
  canvas: HTMLCanvasElement,
  fragmentSource: string,
  preferWebgl2: boolean,
): ProgramBundle {
  const attrs: WebGLContextAttributes = {
    alpha: true,
    antialias: false,
    depth: false,
    stencil: false,
    premultipliedAlpha: true,
  };

  let gl: WebGLRenderingContext | WebGL2RenderingContext | null = null;
  let isWebgl2 = false;

  if (preferWebgl2) {
    gl = canvas.getContext("webgl2", attrs) as WebGL2RenderingContext | null;
    isWebgl2 = !!gl;
  }
  if (!gl) {
    const gl1 = canvas.getContext("webgl", attrs) ?? canvas.getContext("experimental-webgl", attrs);
    gl = gl1 as WebGLRenderingContext | null;
    isWebgl2 = false;
  }
  if (!gl) {
    throw new Error("useFragmentShader: WebGL unavailable");
  }

  const fragSrc = wrapFragmentSource(fragmentSource, isWebgl2);
  const vertSrc = isWebgl2 ? DEFAULT_VERTEX : DEFAULT_VERTEX_WEBGL1;

  const vert = compileShader(gl, gl.VERTEX_SHADER, vertSrc);
  const frag = compileShader(gl, gl.FRAGMENT_SHADER, fragSrc);
  const program = linkProgram(gl, vert, frag);
  gl.deleteShader(vert);
  gl.deleteShader(frag);

  let buffer: WebGLBuffer | null = null;
  if (!isWebgl2) {
    buffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
    gl.bufferData(
      gl.ARRAY_BUFFER,
      new Float32Array([-1, -1, 3, -1, -1, 3]),
      gl.STATIC_DRAW,
    );
    const loc = gl.getAttribLocation(program, "a_pos");
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
  }

  return { gl, program, isWebgl2, buffer };
}

function resizeCanvas(
  canvas: HTMLCanvasElement,
  gl: WebGLRenderingContext | WebGL2RenderingContext,
  dpr: number,
): void {
  const w = Math.max(1, Math.floor(canvas.clientWidth * dpr));
  const h = Math.max(1, Math.floor(canvas.clientHeight * dpr));
  if (canvas.width !== w || canvas.height !== h) {
    canvas.width = w;
    canvas.height = h;
  }
  gl.viewport(0, 0, canvas.width, canvas.height);
}

function drawFrame(
  bundle: ProgramBundle,
  time: number,
): void {
  const { gl, program, isWebgl2 } = bundle;
  gl.useProgram(program);
  const resLoc = gl.getUniformLocation(program, "u_resolution");
  const timeLoc = gl.getUniformLocation(program, "u_time");
  if (resLoc) gl.uniform2f(resLoc, gl.canvas.width, gl.canvas.height);
  if (timeLoc) gl.uniform1f(timeLoc, time);
  gl.drawArrays(gl.TRIANGLES, 0, 3);
  void isWebgl2;
}

/**
 * Next-compatible fragment-shader helper.
 *
 * Compiles/links a fullscreen-triangle program and draws into a canvas ref.
 * Fragment source may be a full shader or a `shade(uv, time)` body snippet
 * (auto-wrapped — similar to Next.js shader island patterns).
 *
 * ```tsx
 * function Plasma() {
 *   const { canvasRef } = useFragmentShader(`
 *     vec4 shade(vec2 uv, float t) {
 *       return vec4(uv, 0.5 + 0.5 * sin(t), 1.0);
 *     }
 *   `);
 *   return <canvas ref={canvasRef} style={{ width: "100%", height: "100%" }} />;
 * }
 * ```
 */
export function useFragmentShader(
  source: string,
  options: UseFragmentShaderOptions = {},
): FragmentShaderHandle {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const bundleRef = useRef<ProgramBundle | null>(null);
  const sourceRef = useRef(source);
  const glRef = useRef<WebGLRenderingContext | WebGL2RenderingContext | null>(null);
  const optsRef = useRef(options);
  optsRef.current = options;

  const setSource = (next: string) => {
    sourceRef.current = next;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const preferWebgl2 = optsRef.current.webgl2 !== false;
    if (bundleRef.current) {
      bundleRef.current.gl.deleteProgram(bundleRef.current.program);
      if (bundleRef.current.buffer) {
        bundleRef.current.gl.deleteBuffer(bundleRef.current.buffer);
      }
    }
    const bundle = buildProgram(canvas, next, preferWebgl2);
    bundleRef.current = bundle;
    glRef.current = bundle.gl;
  };

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const preferWebgl2 = options.webgl2 !== false;
    const dpr = options.dpr ?? (typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1);
    const animate = options.animate !== false;

    let bundle: ProgramBundle;
    try {
      bundle = buildProgram(canvas, sourceRef.current, preferWebgl2);
    } catch (err) {
      console.error(err);
      return;
    }
    bundleRef.current = bundle;
    glRef.current = bundle.gl;

    const start = performance.now();
    let frame = 0;
    let alive = true;

    const tick = () => {
      if (!alive || !bundleRef.current) return;
      const b = bundleRef.current;
      resizeCanvas(canvas, b.gl, dpr);
      const t = (performance.now() - start) / 1000;
      drawFrame(b, t);
      optsRef.current.onFrame?.(t, b.gl);
      if (animate) frame = requestAnimationFrame(tick);
    };

    tick();

    return () => {
      alive = false;
      cancelAnimationFrame(frame);
      if (bundleRef.current) {
        bundleRef.current.gl.deleteProgram(bundleRef.current.program);
        if (bundleRef.current.buffer) {
          bundleRef.current.gl.deleteBuffer(bundleRef.current.buffer);
        }
        bundleRef.current = null;
      }
      glRef.current = null;
    };
    // Re-init when source identity changes from the hook arg
  }, [source, options.webgl2, options.dpr, options.animate]);

  return {
    canvasRef,
    get gl() {
      return glRef.current;
    },
    setSource,
  };
}

/** Pure helper for tests / SSR: compile wrap without a GL context. */
export function createFullscreenFragment(source: string, webgl2 = true): {
  vertex: string;
  fragment: string;
} {
  return {
    vertex: webgl2 ? DEFAULT_VERTEX : DEFAULT_VERTEX_WEBGL1,
    fragment: wrapFragmentSource(source, webgl2),
  };
}
