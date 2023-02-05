/** Script that bundles JS client code. */

import path, { dirname } from 'node:path'
import esbuild from 'esbuild'
import { require_env, require_env_resolved_path } from '../../utils.js'
import aliasPlugin from 'esbuild-plugin-alias'
import { fileURLToPath } from 'node:url'

// ===================================================
// === Constants provided through the environment. ===
// ===================================================

/** Output directory for bundled client files. */
const outdir = path.join(require_env_resolved_path('ENSO_BUILD_IDE'), 'client')

/** Path to the project manager executable relative to the PM bundle root. */
const projectManagerInBundlePath = require_env('ENSO_BUILD_PROJECT_MANAGER_IN_BUNDLE_PATH')

/** Version of the Engine (backend) that is bundled along with this client build. */
const bundledEngineVersion = require_env('ENSO_BUILD_IDE_BUNDLED_ENGINE_VERSION')

export const thisPath = path.resolve(dirname(fileURLToPath(import.meta.url)))

/** The main JS bundle to load WASM and JS wasm-pack bundles. */
export const ensogl_app_path = `/Users/wdanilo/Dev/enso/target/ensogl-pack/dist/index.js`

// ================
// === Bundling ===
// ================

const bundlerOptions: esbuild.BuildOptions = {
    bundle: true,
    outdir,
    entryPoints: ['src/index.ts', 'src/preload.cjs'],
    outbase: 'src',
    plugins: [aliasPlugin({ ensogl_app: ensogl_app_path })],
    format: 'cjs',
    outExtension: { '.js': '.cjs' },
    platform: 'node',
    define: {
        BUNDLED_ENGINE_VERSION: JSON.stringify(bundledEngineVersion),
        PROJECT_MANAGER_IN_BUNDLE_PATH: JSON.stringify(projectManagerInBundlePath),
    },
    sourcemap: true,
    external: ['electron'],
}

await esbuild.build(bundlerOptions)
