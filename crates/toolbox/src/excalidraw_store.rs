use gpui_excalidraw::Editor;

use crate::config_store;

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    crate::TOKIO_RUNTIME.block_on(future)
}

pub fn install_scene_store(editor: &mut Editor) {
    editor.set_scene_store(
        |name, json| block_on(config_store::save_excalidraw_doc(name, json)),
        || {
            block_on(config_store::load_excalidraw_docs()).map(|docs| {
                docs.into_iter()
                    .map(|doc| (doc.name, doc.elements_json))
                    .collect()
            })
        },
    );
}
