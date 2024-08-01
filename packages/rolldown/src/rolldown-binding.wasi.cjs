/* eslint-disable */
/* prettier-ignore */

/* auto-generated by NAPI-RS */

const __nodeFs = require('node:fs')
const __nodePath = require('node:path')
const { WASI: __nodeWASI } = require('node:wasi')
const { Worker } = require('node:worker_threads')

const {
  instantiateNapiModuleSync: __emnapiInstantiateNapiModuleSync,
  getDefaultContext: __emnapiGetDefaultContext,
  createOnMessage: __wasmCreateOnMessageForFsProxy,
} = require('@napi-rs/wasm-runtime')

const __rootDir = __nodePath.parse(process.cwd()).root

const __wasi = new __nodeWASI({
  version: 'preview1',
  env: process.env,
  preopens: {
    [__rootDir]: __rootDir,
  }
})

const __emnapiContext = __emnapiGetDefaultContext()

const __sharedMemory = new WebAssembly.Memory({
  initial: 16384,
  maximum: 65536,
  shared: true,
})

let __wasmFilePath = __nodePath.join(__dirname, 'rolldown-binding.wasm32-wasi.wasm')
const __wasmDebugFilePath = __nodePath.join(__dirname, 'rolldown-binding.wasm32-wasi.debug.wasm')

if (__nodeFs.existsSync(__wasmDebugFilePath)) {
  __wasmFilePath = __wasmDebugFilePath
} else if (!__nodeFs.existsSync(__wasmFilePath)) {
  try {
    __wasmFilePath = __nodePath.resolve('@rolldown/binding-wasm32-wasi')
  } catch {
    throw new Error('Cannot find rolldown-binding.wasm32-wasi.wasm file, and @rolldown/binding-wasm32-wasi package is not installed.')
  }
}

const { instance: __napiInstance, module: __wasiModule, napiModule: __napiModule } = __emnapiInstantiateNapiModuleSync(__nodeFs.readFileSync(__wasmFilePath), {
  context: __emnapiContext,
  asyncWorkPoolSize: (function() {
    const threadsSizeFromEnv = Number(process.env.NAPI_RS_ASYNC_WORK_POOL_SIZE ?? process.env.UV_THREADPOOL_SIZE)
    // NaN > 0 is false
    if (threadsSizeFromEnv > 0) {
      return threadsSizeFromEnv
    } else {
      return 4
    }
  })(),
  wasi: __wasi,
  onCreateWorker() {
    const worker = new Worker(__nodePath.join(__dirname, 'wasi-worker.mjs'), {
      env: process.env,
      execArgv: ['--experimental-wasi-unstable-preview1'],
    })
    worker.onmessage = ({ data }) => {
      __wasmCreateOnMessageForFsProxy(__nodeFs)(data)
    }
    return worker
  },
  overwriteImports(importObject) {
    importObject.env = {
      ...importObject.env,
      ...importObject.napi,
      ...importObject.emnapi,
      memory: __sharedMemory,
    }
    return importObject
  },
  beforeInit({ instance }) {
    __napi_rs_initialize_modules(instance)
  }
})

function __napi_rs_initialize_modules(__napiInstance) {
  __napiInstance.exports['__napi_register__IsolatedDeclarationsResult_struct_0']?.()
  __napiInstance.exports['__napi_register__isolated_declaration_1']?.()
  __napiInstance.exports['__napi_register__TypeScriptBindingOptions_struct_2']?.()
  __napiInstance.exports['__napi_register__ReactBindingOptions_struct_3']?.()
  __napiInstance.exports['__napi_register__ArrowFunctionsBindingOptions_struct_4']?.()
  __napiInstance.exports['__napi_register__ES2015BindingOptions_struct_5']?.()
  __napiInstance.exports['__napi_register__TransformOptions_struct_6']?.()
  __napiInstance.exports['__napi_register__Sourcemap_struct_7']?.()
  __napiInstance.exports['__napi_register__TransformResult_struct_8']?.()
  __napiInstance.exports['__napi_register__transform_9']?.()
  __napiInstance.exports['__napi_register__Bundler_struct_0']?.()
  __napiInstance.exports['__napi_register__Bundler_impl_5']?.()
  __napiInstance.exports['__napi_register__BindingInputItem_struct_6']?.()
  __napiInstance.exports['__napi_register__BindingResolveOptions_struct_7']?.()
  __napiInstance.exports['__napi_register__BindingTreeshake_struct_8']?.()
  __napiInstance.exports['__napi_register__BindingInputOptions_struct_9']?.()
  __napiInstance.exports['__napi_register__BindingOutputOptions_struct_10']?.()
  __napiInstance.exports['__napi_register__BindingPluginContext_struct_11']?.()
  __napiInstance.exports['__napi_register__BindingPluginContext_impl_17']?.()
  __napiInstance.exports['__napi_register__BindingPluginContextResolvedId_struct_18']?.()
  __napiInstance.exports['__napi_register__BindingPluginOptions_struct_19']?.()
  __napiInstance.exports['__napi_register__BindingPluginWithIndex_struct_20']?.()
  __napiInstance.exports['__napi_register__BindingTransformPluginContext_struct_21']?.()
  __napiInstance.exports['__napi_register__BindingTransformPluginContext_impl_23']?.()
  __napiInstance.exports['__napi_register__BindingAssetSource_struct_24']?.()
  __napiInstance.exports['__napi_register__BindingEmittedAsset_struct_25']?.()
  __napiInstance.exports['__napi_register__BindingHookLoadOutput_struct_26']?.()
  __napiInstance.exports['__napi_register__BindingHookRenderChunkOutput_struct_27']?.()
  __napiInstance.exports['__napi_register__BindingHookResolveIdExtraOptions_struct_28']?.()
  __napiInstance.exports['__napi_register__BindingHookResolveIdOutput_struct_29']?.()
  __napiInstance.exports['__napi_register__BindingHookSideEffects_30']?.()
  __napiInstance.exports['__napi_register__BindingHookTransformOutput_struct_31']?.()
  __napiInstance.exports['__napi_register__BindingPluginContextResolveOptions_struct_32']?.()
  __napiInstance.exports['__napi_register__BindingBuiltinPlugin_struct_33']?.()
  __napiInstance.exports['__napi_register__BindingBuiltinPluginName_34']?.()
  __napiInstance.exports['__napi_register__BindingGlobImportPluginConfig_struct_35']?.()
  __napiInstance.exports['__napi_register__BindingManifestPluginConfig_struct_36']?.()
  __napiInstance.exports['__napi_register__ParallelJsPluginRegistry_struct_37']?.()
  __napiInstance.exports['__napi_register__ParallelJsPluginRegistry_impl_39']?.()
  __napiInstance.exports['__napi_register__register_plugins_40']?.()
  __napiInstance.exports['__napi_register__BindingLog_struct_41']?.()
  __napiInstance.exports['__napi_register__BindingLogLevel_42']?.()
  __napiInstance.exports['__napi_register__BindingModuleInfo_struct_43']?.()
  __napiInstance.exports['__napi_register__BindingModuleInfo_impl_45']?.()
  __napiInstance.exports['__napi_register__BindingOutputAsset_struct_46']?.()
  __napiInstance.exports['__napi_register__BindingOutputAsset_impl_51']?.()
  __napiInstance.exports['__napi_register__BindingOutputChunk_struct_52']?.()
  __napiInstance.exports['__napi_register__BindingOutputChunk_impl_70']?.()
  __napiInstance.exports['__napi_register__BindingOutputs_struct_71']?.()
  __napiInstance.exports['__napi_register__BindingOutputs_impl_75']?.()
  __napiInstance.exports['__napi_register__FinalBindingOutputs_struct_76']?.()
  __napiInstance.exports['__napi_register__FinalBindingOutputs_impl_79']?.()
  __napiInstance.exports['__napi_register__RenderedChunk_struct_80']?.()
  __napiInstance.exports['__napi_register__BindingRenderedModule_struct_81']?.()
  __napiInstance.exports['__napi_register__AliasItem_struct_82']?.()
  __napiInstance.exports['__napi_register__BindingSourcemap_struct_83']?.()
  __napiInstance.exports['__napi_register__BindingJsonSourcemap_struct_84']?.()
}
module.exports.BindingLog = __napiModule.exports.BindingLog
module.exports.BindingModuleInfo = __napiModule.exports.BindingModuleInfo
module.exports.BindingOutputAsset = __napiModule.exports.BindingOutputAsset
module.exports.BindingOutputChunk = __napiModule.exports.BindingOutputChunk
module.exports.BindingOutputs = __napiModule.exports.BindingOutputs
module.exports.BindingPluginContext = __napiModule.exports.BindingPluginContext
module.exports.BindingTransformPluginContext = __napiModule.exports.BindingTransformPluginContext
module.exports.Bundler = __napiModule.exports.Bundler
module.exports.FinalBindingOutputs = __napiModule.exports.FinalBindingOutputs
module.exports.ParallelJsPluginRegistry = __napiModule.exports.ParallelJsPluginRegistry
module.exports.BindingBuiltinPluginName = __napiModule.exports.BindingBuiltinPluginName
module.exports.BindingHookSideEffects = __napiModule.exports.BindingHookSideEffects
module.exports.BindingLogLevel = __napiModule.exports.BindingLogLevel
module.exports.isolatedDeclaration = __napiModule.exports.isolatedDeclaration
module.exports.registerPlugins = __napiModule.exports.registerPlugins
module.exports.transform = __napiModule.exports.transform
