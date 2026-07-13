/// <reference types="vite/client" />

// The `?inline` CSS import yields the compiled stylesheet as a STRING (Vite's
// `?inline` query), which the SDK's `defineRemote({ styles })` attaches scoped
// under the ext root (never `document.head`). `vite/client` already types the
// bare `*.css` import; declare the `?inline` variant explicitly.
declare module "*.css?inline" {
  const css: string;
  export default css;
}
