/**
 * moonshine — lightweight React framework runtime for crepuscularity.
 *
 * Public API surface.
 */

export { createMoonshineApp } from "./create-app";
export type { MoonshineApp, MoonshineAppOptions } from "./create-app";

export {
  batch,
  createMemo,
  createSignal,
  createStore,
  useSignal,
  useStore,
} from "./signal";
export type { Memo, Signal, StoreSetter } from "./signal";

export {
  Link,
  MoonshineRouter,
  getLocation,
  matchPath,
  matchRoutes,
  navigate,
  useLocation,
  useNavigate,
  useParams,
} from "./router";
export type {
  MoonshineRouterProps,
  RouteDefinition,
  RouteMatch,
  RouteParams,
} from "./router";

export {
  createMoonshineServer,
  definePage,
  handleMoonshineRequest,
  resolvePage,
  toMoonshineRequest,
} from "./server";
export type {
  MoonshinePageModule,
  MoonshineRequest,
  MoonshineServer,
  MoonshineServerOptions,
} from "./server";

export {
  createFullscreenFragment,
  useFragmentShader,
  wrapFragmentSource,
} from "./shaders";
export type { FragmentShaderHandle, UseFragmentShaderOptions } from "./shaders";
