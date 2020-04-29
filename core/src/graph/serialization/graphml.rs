use crate::{
    annostorage::ValueSearch,
    graph::{Graph, ANNIS_NS, NODE_TYPE},
    types::{AnnoKey, ComponentType},
    util::join_qname,
};
use anyhow::Result;
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use std::collections::BTreeMap;

fn export_annotation_keys<CT: ComponentType, W: std::io::Write>(
    graph: &Graph<CT>,
    writer: &mut Writer<W>,
) -> Result<BTreeMap<AnnoKey, String>> {
    let mut key_id_mapping = BTreeMap::new();
    let mut id_counter = 0;

    // Export node annotations
    for key in graph.get_node_annos().annotation_keys() {
        let mut key_start = BytesStart::borrowed_name("key".as_bytes());
        let qname = join_qname(&key.ns, &key.name);

        let id = key_id_mapping.entry(key).or_insert_with(|| {
            let result = format!("k{}", id_counter);
            id_counter += 1;
            result
        });

        key_start.push_attribute(("id", id.as_str()));
        key_start.push_attribute(("for", "node"));
        key_start.push_attribute(("attr.name", qname.as_str()));
        key_start.push_attribute(("attr.type", "string"));

        writer.write_event(Event::Empty(key_start))?;
    }
    Ok(key_id_mapping)
}

fn export_nodes<CT: ComponentType, W: std::io::Write>(
    graph: &Graph<CT>,
    writer: &mut Writer<W>,
    key_id_mapping: &BTreeMap<AnnoKey, String>,
) -> Result<()> {
    for m in graph
        .get_node_annos()
        .exact_anno_search(Some(ANNIS_NS), NODE_TYPE, ValueSearch::Any)
    {
        let mut node_start = BytesStart::borrowed_name("node".as_bytes());

        node_start.push_attribute(("id", m.node.to_string().as_str()));
        writer.write_event(Event::Start(node_start))?;
        // Write all annotations of the node as "data" element
        for anno in graph.get_node_annos().get_annotations_for_item(&m.node) {
            let mut data_start = BytesStart::borrowed_name(b"data");

            let key_id = key_id_mapping.get(&anno.key).ok_or_else(|| {
                anyhow!(
                    "Could not find annotation key ID for {:?} when mapping to GraphML",
                    &anno.key
                )
            })?;

            data_start.push_attribute(("key", key_id.as_str()));
            writer.write_event(Event::Start(data_start))?;
            // Add the annotation value as internal text node
            writer.write_event(Event::Text(BytesText::from_plain(anno.val.as_bytes())))?;
            writer.write_event(Event::End(BytesEnd::borrowed(b"data")))?;
        }

        writer.write_event(Event::End(BytesEnd::borrowed(b"node")))?;
    }
    Ok(())
}

fn export_edges<CT: ComponentType, W: std::io::Write>(
    graph: &Graph<CT>,
    writer: &mut Writer<W>,
) -> Result<()> {
    todo!()
}

pub fn export<CT: ComponentType, W: std::io::Write>(graph: &Graph<CT>, output: W) -> Result<()> {
    let mut writer = Writer::new_with_indent(output, b' ', 4);

    // Add XML declaration
    let xml_decl = BytesDecl::new(b"1.0", Some(b"UTF-8"), None);
    writer.write_event(Event::Decl(xml_decl))?;

    // Always write the root element
    writer.write_event(Event::Start(BytesStart::borrowed_name(b"graphml")))?;

    // Define all valid annotation ns/name pairs
    let key_id_mapping = export_annotation_keys(graph, &mut writer)?;

    // We are writing a single graph
    let mut graph_start = BytesStart::borrowed_name("graph".as_bytes());
    graph_start.push_attribute(("edgedefault", "directed"));
    writer.write_event(Event::Start(graph_start))?;

    // Write out all nodes
    export_nodes(graph, &mut writer, &key_id_mapping)?;

    // Write out all edges
    //export_edges(graph, &mut writer)?;

    writer.write_event(Event::End(BytesEnd::borrowed(b"graph")))?;
    writer.write_event(Event::End(BytesEnd::borrowed(b"graphml")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::{GraphUpdate, UpdateEvent},
        types::DefaultComponentType,
    };
    #[test]
    fn export_graphml() {
        // Create a sample graph using the simple type
        let mut u = GraphUpdate::new();
        u.add_event(UpdateEvent::AddNode {
            node_name: "n0".to_string(),
            node_type: "node".to_string(),
        })
        .unwrap();
        u.add_event(UpdateEvent::AddNode {
            node_name: "n1".to_string(),
            node_type: "node".to_string(),
        })
        .unwrap();

        let mut g: Graph<DefaultComponentType> = Graph::new(false).unwrap();
        g.apply_update(&mut u, |_| {}).unwrap();

        // export to GraphML, read generated XML and compare it
        let mut xml_data: Vec<u8> = Vec::default();
        export(&g, &mut xml_data).unwrap();
        let expected = include_str!("output_example.xml");
        let actual = String::from_utf8(xml_data).unwrap();
        assert_eq!(expected, actual);
    }
}
