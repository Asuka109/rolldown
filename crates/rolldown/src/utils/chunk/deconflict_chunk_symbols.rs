use std::borrow::Cow;

use crate::{stages::link_stage::LinkStageOutput, utils::renamer::Renamer};
use rolldown_common::{Chunk, ChunkKind, OutputFormat};
use rolldown_rstr::{Rstr, ToRstr};

#[tracing::instrument(level = "trace", skip_all)]
pub fn deconflict_chunk_symbols(
  chunk: &mut Chunk,
  link_output: &LinkStageOutput,
  format: &OutputFormat,
) {
  let mut renamer =
    Renamer::new(&link_output.symbols, link_output.module_table.modules.len(), format);
  chunk
    .modules
    .iter()
    .copied()
    .filter_map(|id| link_output.module_table.modules[id].as_ecma())
    .flat_map(|m| m.scope.root_unresolved_references().keys().map(Cow::Borrowed))
    .for_each(|name| {
      // global names should be reserved
      renamer.reserve(Cow::Owned(name.to_rstr()));
    });
  // `export_default` is used for create temp variable to export a commonjs import binding
  renamer.reserve(Cow::Owned(Rstr::from("export_default")));

  // Though, those symbols in `imports_from_other_chunks` doesn't belong to this chunk, but in the final output, they still behave
  // like declared in this chunk. This is because we need to generate import statements in this chunk to import symbols from other
  // statements. Those `import {...} from './other-chunk.js'` will declared these outside symbols in this chunk, so symbols that
  // point to them can be resolved in runtime.
  // So we add them in the deconflict process to generate conflict-less names in this chunk.
  chunk.imports_from_other_chunks.iter().flat_map(|(_, items)| items.iter()).for_each(|item| {
    renamer.add_top_level_symbol(item.import_ref);
  });

  chunk.require_binding_names_for_other_chunks.values_mut().for_each(|name_hint| {
    *name_hint = renamer.create_conflictless_top_level_name(&format!("require_{name_hint}"));
  });
  match chunk.kind {
    ChunkKind::EntryPoint { module, .. } => {
      let meta = &link_output.metas[module];
      meta
        .require_bindings_for_star_exports
        .iter()
        .for_each(|(_module, binding_ref)| renamer.add_top_level_symbol(*binding_ref));
    }
    ChunkKind::Common => {}
  }

  chunk
    .modules
    .iter()
    .copied()
    // Starts with entry module
    .rev()
    .filter_map(|id| link_output.module_table.modules[id].as_ecma())
    .for_each(|module| {
      module
        .stmt_infos
        .iter()
        .filter(|stmt_info| stmt_info.is_included)
        .flat_map(|stmt_info| stmt_info.declared_symbols.iter().copied())
        .for_each(|symbol_ref| {
          renamer.add_top_level_symbol(symbol_ref);
        });
    });

  // rename non-top-level names
  renamer.rename_non_top_level_symbol(&chunk.modules, &link_output.module_table.modules);

  chunk.canonical_names = renamer.into_canonical_names();
}
