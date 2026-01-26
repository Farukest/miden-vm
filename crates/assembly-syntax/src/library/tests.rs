//! Tests for Library serialization

use alloc::{collections::BTreeMap, sync::Arc, vec};

use miden_core::{
    Operation,
    mast::{BasicBlockNodeBuilder, MastForest, MastForestContributor},
    utils::{Deserializable, Serializable, SliceReader},
};

use super::{Library, LibraryExport, ProcedureExport};
use crate::ast::{Attribute, AttributeSet, Ident, Path, PathBuf};

/// Test that procedure attributes are preserved after Library serialization/deserialization.
///
/// This test demonstrates issue #2608: Library serialization does not preserve procedure attributes.
/// See: https://github.com/0xMiden/miden-vm/issues/2608
#[test]
fn library_serialization_preserves_procedure_attributes() {
    // Create a simple MAST forest with one procedure
    let mut mast_forest = MastForest::new();
    let node_id = BasicBlockNodeBuilder::new(vec![Operation::Add, Operation::Mul], vec![])
        .add_to_forest(&mut mast_forest)
        .unwrap();
    mast_forest.make_root(node_id);

    // Create an attribute (e.g., @note_script)
    let attr_name = Ident::new("note_script").unwrap();
    let attribute = Attribute::Marker(attr_name);
    let mut attributes = AttributeSet::default();
    attributes.insert(attribute);

    // Verify the attribute was added
    assert!(attributes.has("note_script"), "attribute should be present before serialization");

    // Create a procedure export with the attribute
    let proc_path: Arc<Path> =
        Arc::from(PathBuf::new("test::my_proc").expect("valid path").into_boxed_path());
    let procedure_export = ProcedureExport {
        node: node_id,
        path: proc_path.clone(),
        signature: None,
        attributes: attributes.clone(),
    };

    // Verify the procedure export has the attribute
    assert!(
        procedure_export.attributes.has("note_script"),
        "procedure export should have attribute before serialization"
    );

    // Create the library
    let mut exports = BTreeMap::new();
    exports.insert(proc_path.clone(), LibraryExport::Procedure(procedure_export));
    let library = Library::new(Arc::new(mast_forest), exports).unwrap();

    // Verify the library has the attribute before serialization
    let export_before = library.exports().next().unwrap();
    let proc_before = export_before.unwrap_procedure();
    assert!(
        proc_before.attributes.has("note_script"),
        "library export should have attribute before serialization"
    );

    // Serialize the library
    let mut bytes = vec![];
    library.write_into(&mut bytes);

    // Deserialize the library
    let mut reader = SliceReader::new(&bytes);
    let deserialized_library = Library::read_from(&mut reader).unwrap();

    // Verify the deserialized library has the attribute
    let export_after = deserialized_library.exports().next().unwrap();
    let proc_after = export_after.unwrap_procedure();

    // This assertion will FAIL due to issue #2608 - attributes are not serialized
    assert!(
        proc_after.attributes.has("note_script"),
        "ISSUE #2608: library export should have attribute after deserialization, but attributes \
         are lost during serialization. Expected attributes: {:?}, Got: {:?}",
        proc_before.attributes,
        proc_after.attributes
    );
}
