use std::{
  hash::{DefaultHasher, Hash, Hasher},
  sync::Arc,
};

use futures::future::try_join_all;
use index_vec::IndexVec;
use rolldown_common::{
  ChunkId, ChunkKind, FileNameRenderOptions, Output, OutputAsset, OutputChunk, PreliminaryFilename,
  SourceMapType,
};
use rolldown_error::BuildError;
use rolldown_plugin::SharedPluginDriver;
use rolldown_utils::rayon::{ParallelBridge, ParallelIterator};
use rustc_hash::FxHashSet;

use crate::{
  chunk_graph::ChunkGraph,
  error::BatchedResult,
  finalizer::FinalizerContext,
  stages::link_stage::LinkStageOutput,
  utils::{
    chunk::{
      deconflict_chunk_symbols::deconflict_chunk_symbols,
      render_chunk::{render_chunk, ChunkRenderReturn},
    },
    extract_hash_placeholder::{
      extract_hash_placeholder, generate_facade_replacement_of_hash_placeholder,
    },
    finalize_normal_module, is_in_rust_test_mode,
    render_chunks::render_chunks,
  },
  SharedOptions,
};

mod code_splitting;
mod compute_cross_chunk_links;

pub struct GenerateStage<'a> {
  link_output: &'a mut LinkStageOutput,
  options: &'a SharedOptions,
  plugin_driver: &'a SharedPluginDriver,
}

impl<'a> GenerateStage<'a> {
  pub fn new(
    link_output: &'a mut LinkStageOutput,
    options: &'a SharedOptions,
    plugin_driver: &'a SharedPluginDriver,
  ) -> Self {
    Self { link_output, options, plugin_driver }
  }

  #[tracing::instrument(skip_all)]
  pub async fn generate(&mut self) -> BatchedResult<Vec<Output>> {
    tracing::info!("Start bundle stage");
    let mut chunk_graph = self.generate_chunks();

    self.generate_chunk_preliminary_filenames(&mut chunk_graph);
    tracing::info!("generate_chunk_preliminary_filenames");

    self.compute_cross_chunk_links(&mut chunk_graph);
    tracing::info!("compute_cross_chunk_links");

    chunk_graph.chunks.iter_mut().par_bridge().for_each(|chunk| {
      deconflict_chunk_symbols(chunk, self.link_output);
    });

    let ast_table_iter = self.link_output.ast_table.iter_mut_enumerated();
    ast_table_iter
      .par_bridge()
      .filter(|(id, _)| self.link_output.module_table.normal_modules[*id].is_included)
      .for_each(|(id, ast)| {
        let module = &self.link_output.module_table.normal_modules[id];
        let chunk_id = chunk_graph.module_to_chunk[module.id].unwrap();
        let chunk = &chunk_graph.chunks[chunk_id];
        let linking_info = &self.link_output.metas[module.id];
        finalize_normal_module(
          module,
          FinalizerContext {
            canonical_names: &chunk.canonical_names,
            id: module.id,
            symbols: &self.link_output.symbols,
            linking_info,
            module,
            modules: &self.link_output.module_table.normal_modules,
            linking_infos: &self.link_output.metas,
            runtime: &self.link_output.runtime,
            chunk_graph: &chunk_graph,
          },
          ast,
        );
      });
    tracing::info!("finalizing modules");

    let chunks = try_join_all(
      chunk_graph
        .chunks
        .iter()
        .map(|c| async { render_chunk(c, self.options, self.link_output, &chunk_graph).await }),
    )
    .await?;

    let mut chunks = render_chunks(self.plugin_driver, chunks).await?;

    let base_hash_state = chunks
      .iter()
      .map(|chunk| {
        // TODO: use a better hash function
        let mut state = DefaultHasher::default();
        chunk.code.hash(&mut state);
        state
      })
      .collect::<IndexVec<ChunkId, _>>();

    // calculate the final hash of each chunk by traverse the chunk graph

    let final_hashes = base_hash_state
      .into_iter()
      .map(|state| state.finish().to_string())
      .collect::<IndexVec<ChunkId, _>>();

    chunk_graph.chunks.iter_mut().zip(chunks.iter_mut()).zip(final_hashes).for_each(
      |((chunk, chunk_render_return), hash)| {
        let preliminary_filename_raw =
          chunk.preliminary_filename.as_ref().expect("should have file name").as_str();
        let filename = chunk
          .preliminary_filename
          .as_ref()
          .expect("should have file name")
          .finalize(&hash)
          .into_owned();

        chunk_render_return.rendered_chunk.file_name = filename;
        // TODO replace code
        // chunk_render_return.code = chunk_render_return.code.replace(from, to)
      },
    );

    let mut assets = vec![];
    for ChunkRenderReturn { mut map, rendered_chunk, mut code } in chunks {
      if let Some(map) = map.as_mut() {
        map.set_file(&rendered_chunk.file_name);
        match self.options.sourcemap {
          SourceMapType::File => {
            let map_file_name = format!("{}.map", rendered_chunk.file_name);
            assets.push(Output::Asset(Arc::new(OutputAsset {
              file_name: map_file_name.clone(),
              source: map.to_json_string().map_err(BuildError::sourcemap_error)?,
            })));
            code.push_str(&format!("\n//# sourceMappingURL={map_file_name}"));
          }
          SourceMapType::Inline => {
            let data_url = map.to_data_url().map_err(BuildError::sourcemap_error)?;
            code.push_str(&format!("\n//# sourceMappingURL={data_url}"));
          }
          SourceMapType::Hidden => {}
        }
      }
      let sourcemap_file_name = map.as_ref().map(|_| format!("{}.map", rendered_chunk.file_name));
      assets.push(Output::Chunk(Arc::new(OutputChunk {
        file_name: rendered_chunk.file_name,
        code,
        is_entry: rendered_chunk.is_entry,
        is_dynamic_entry: rendered_chunk.is_dynamic_entry,
        facade_module_id: rendered_chunk.facade_module_id,
        modules: rendered_chunk.modules,
        exports: rendered_chunk.exports,
        module_ids: rendered_chunk.module_ids,
        map,
        sourcemap_file_name,
      })));
    }

    tracing::info!("rendered chunks");

    Ok(assets)
  }

  fn generate_chunk_preliminary_filenames(&self, chunk_graph: &mut ChunkGraph) {
    let mut used_chunk_names = FxHashSet::default();
    chunk_graph.chunks.iter_mut().for_each(|chunk| {
      let runtime_id = self.link_output.runtime.id();

      let file_name_tmp = chunk.file_name_template(self.options);
      let chunk_name =
        if is_in_rust_test_mode() && chunk.modules.first().copied() == Some(runtime_id) {
          "$runtime$".to_string()
        } else {
          chunk.name.clone().unwrap_or_else(|| {
            let module_id =
              if let ChunkKind::EntryPoint { module: entry_module_id, is_user_defined, .. } =
                &chunk.kind
              {
                debug_assert!(
                  !*is_user_defined,
                  "User-defined entry point should always have a name"
                );
                *entry_module_id
              } else {
                // TODO: we currently use the first executed module to calculate the chunk name for common chunks
                // This is not perfect, should investigate more to find a better solution
                chunk.modules.first().copied().unwrap()
              };
            let module = &self.link_output.module_table.normal_modules[module_id];
            module.resource_id.expect_file().unique(&self.options.cwd)
          })
        };

      let mut chunk_name = chunk_name;
      while used_chunk_names.contains(&chunk_name) {
        chunk_name = format!("{}-{}", chunk_name, used_chunk_names.len());
      }
      used_chunk_names.insert(chunk_name.clone());

      let extracted_hash = extract_hash_placeholder(file_name_tmp.template());

      let preliminary = file_name_tmp.render(&FileNameRenderOptions {
        name: Some(&chunk_name),
        hash: extracted_hash
          .as_ref()
          .map(|extracted| generate_facade_replacement_of_hash_placeholder(extracted))
          .as_deref(),
      });

      chunk.preliminary_filename = Some(PreliminaryFilename {
        filename: preliminary,
        hash_placeholder: extracted_hash.map(|v| v.pattern),
      });
    });
  }
}
