/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_WSS_URI?: string;
  readonly VITE_HTTP_URI?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
