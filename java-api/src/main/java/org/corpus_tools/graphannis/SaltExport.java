/*
 * Copyright 2017 Thomas Krause.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
package org.corpus_tools.graphannis;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.LinkedList;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

import org.apache.commons.lang3.tuple.ImmutablePair;
import org.apache.commons.lang3.tuple.Pair;
import org.corpus_tools.graphannis.capi.AnnisAnnotation;
import org.corpus_tools.graphannis.capi.AnnisComponentType;
import org.corpus_tools.graphannis.capi.AnnisEdge;
import org.corpus_tools.graphannis.capi.AnnisString;
import org.corpus_tools.graphannis.capi.CAPI;
import org.corpus_tools.graphannis.capi.CAPI.AnnisComponentConst;
import org.corpus_tools.graphannis.capi.CAPI.AnnisGraphDB;
import org.corpus_tools.graphannis.capi.CAPI.AnnisVec_AnnisComponent;
import org.corpus_tools.graphannis.capi.CAPI.AnnisVec_AnnisEdge;
import org.corpus_tools.graphannis.capi.NodeID;
import org.corpus_tools.graphannis.capi.NodeIDByRef;
import org.corpus_tools.salt.SALT_TYPE;
import org.corpus_tools.salt.SaltFactory;
import org.corpus_tools.salt.common.SCorpus;
import org.corpus_tools.salt.common.SCorpusGraph;
import org.corpus_tools.salt.common.SDocument;
import org.corpus_tools.salt.common.SDocumentGraph;
import org.corpus_tools.salt.common.SOrderRelation;
import org.corpus_tools.salt.common.SSpan;
import org.corpus_tools.salt.common.STextualDS;
import org.corpus_tools.salt.common.STextualRelation;
import org.corpus_tools.salt.common.STimeline;
import org.corpus_tools.salt.common.STimelineRelation;
import org.corpus_tools.salt.common.SToken;
import org.corpus_tools.salt.core.GraphTraverseHandler;
import org.corpus_tools.salt.core.SAnnotationContainer;
import org.corpus_tools.salt.core.SFeature;
import org.corpus_tools.salt.core.SGraph;
import org.corpus_tools.salt.core.SLayer;
import org.corpus_tools.salt.core.SNode;
import org.corpus_tools.salt.core.SRelation;
import org.corpus_tools.salt.util.SaltUtil;

import com.google.common.base.Objects;
import com.google.common.base.Splitter;
import com.google.common.collect.BiMap;
import com.google.common.collect.HashBiMap;
import com.google.common.collect.Multimap;
import com.google.common.collect.Range;
import com.sun.jna.NativeLong;

/**
 * Allows to extract a Salt-Graph from a database subgraph.
 * 
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class SaltExport {

    private final CAPI.AnnisGraphDB orig;
    private final SDocumentGraph docGraph;
    private final BiMap<Integer, SNode> nodesByID;
    private final Map<Integer, Integer> node2timelinePOT;

    protected SaltExport(AnnisGraphDB orig) {
        this.orig = orig;

        this.docGraph = SaltFactory.createSDocumentGraph();
        this.nodesByID = HashBiMap.create();
        this.node2timelinePOT = new HashMap<>();
    }

    private static void mapLabels(SAnnotationContainer n, Map<Pair<String, String>, String> labels, boolean isMeta) {
        for (Map.Entry<Pair<String, String>, String> e : labels.entrySet()) {
            if ("annis".equals(e.getKey().getKey())) {
                n.createFeature(e.getKey().getKey(), e.getKey().getValue(), e.getValue());
            } else if (isMeta) {
                n.createMetaAnnotation(e.getKey().getKey(), e.getKey().getValue(), e.getValue());
            } else {
                n.createAnnotation(e.getKey().getKey(), e.getKey().getValue(), e.getValue());
            }
        }

    }

    private boolean hasDominanceEdge(NodeIDByRef nID) {

        AnnisVec_AnnisComponent components = CAPI.annis_graph_all_components(orig);
        for (int i = 0; i < CAPI.annis_vec_component_size(components).intValue(); i++) {
            CAPI.AnnisComponentConst c = CAPI.annis_vec_component_get(components, new NativeLong(i));
            if (AnnisComponentType.Dominance == CAPI.annis_component_type(c)) {
                // check if the node has an outgoing edge of this component
                AnnisVec_AnnisEdge outEdges = CAPI.annis_graph_outgoing_edges(orig, new NodeID(nID.getValue()), c);
                if (CAPI.annis_vec_edge_size(outEdges).longValue() > 0) {
                    return true;
                }
            }
        }
        return false;
    }

    private static Map<Pair<String, String>, String> getNodeLabels(CAPI.AnnisGraphDB g, int nID) {
        Map<Pair<String, String>, String> labels = new LinkedHashMap<>();
        CAPI.AnnisVec_AnnisAnnotation annos = CAPI.annis_graph_node_labels(g, new NodeID(nID));
        for (long i = 0; i < CAPI.annis_vec_annotation_size(annos).longValue(); i++) {
            AnnisAnnotation.ByReference a = CAPI.annis_vec_annotation_get(annos, new NativeLong(i));

            String ns = CAPI.annis_graph_str(g, a.key.ns).toString();
            String name = CAPI.annis_graph_str(g, a.key.name).toString();
            String value = CAPI.annis_graph_str(g, a.value).toString();

            if (name != null && value != null) {
                if (ns == null) {
                    labels.put(new ImmutablePair<>("", name), value);
                } else {
                    labels.put(new ImmutablePair<>(ns, name), value);
                }
            }
        }
        annos.dispose();

        return labels;
    }

    private SNode mapNode(NodeIDByRef nID) {
        SNode newNode;

        // get all annotations for the node into a map, also create the node itself
        int copyID = nID.getValue();
        Map<Pair<String, String>, String> labels = getNodeLabels(orig, copyID);

        if (labels.containsKey(new ImmutablePair<>("annis", "tok"))) {
            newNode = SaltFactory.createSToken();
        } else if (hasDominanceEdge(nID)) {
            newNode = SaltFactory.createSStructure();
        } else {
            newNode = SaltFactory.createSSpan();
        }
        newNode.createFeature("annis", "node_id", copyID);

        String nodeName = labels.get(new ImmutablePair<>("annis", "node_name"));
        if (nodeName != null) {
            if (!nodeName.startsWith("salt:/")) {
                nodeName = "salt:/" + nodeName;
            }
            newNode.setId(nodeName);
            // get the name from the ID
            newNode.setName(newNode.getPath().fragment());

        }

        mapLabels(newNode, labels, false);

        return newNode;
    }

    private void mapAndAddEdge(NodeID node, AnnisEdge origEdge, CAPI.AnnisComponentConst component) {
        SNode source = nodesByID.get(origEdge.source.intValue());
        SNode target = nodesByID.get(origEdge.target.intValue());

        String edgeType = null;
        if (source != null && target != null && source != target) {
            AnnisString cName = CAPI.annis_component_name(component);
            if (cName != null) {
                edgeType = cName.toString();
            }
            SRelation<?, ?> rel = null;
            switch (CAPI.annis_component_type(component)) {
            case AnnisComponentType.Dominance:
                if (edgeType == null || edgeType.isEmpty()) {
                    // We don't include edges that have no type if there is an edge
                    // between the same nodes which has a type.
                    AnnisVec_AnnisComponent domComponents = CAPI.annis_graph_all_components_by_type(orig,
                            AnnisComponentType.Dominance);
                    for (int cIdx = 0; cIdx < CAPI.annis_vec_component_size(domComponents).intValue(); cIdx++) {
                        CAPI.AnnisComponentConst dc = CAPI.annis_vec_component_get(domComponents, new NativeLong(cIdx));

                        if (!CAPI.annis_component_name(dc).toString().isEmpty()
                                && !CAPI.annis_component_layer(dc).toString().isEmpty()) {
                            AnnisVec_AnnisEdge outEdges = CAPI.annis_graph_outgoing_edges(orig, origEdge.source, dc);
                            for (int i = 0; i < CAPI.annis_vec_edge_size(outEdges).intValue(); i++) {
                                AnnisEdge outEdge = CAPI.annis_vec_edge_get(outEdges, new NativeLong(i));

                                if (outEdge.target.equals(origEdge.target)) {
                                    // exclude this relation
                                    return;
                                }
                            }
                        }
                    }
                } // end mirror check
                rel = docGraph.createRelation(source, target, SALT_TYPE.SDOMINANCE_RELATION, null);

                break;
            case AnnisComponentType.Pointing:
                rel = docGraph.createRelation(source, target, SALT_TYPE.SPOINTING_RELATION, null);
                break;
            case AnnisComponentType.Ordering:
                rel = docGraph.createRelation(source, target, SALT_TYPE.SORDER_RELATION, null);
                break;
            case AnnisComponentType.Coverage:
                // only add coverage edges in salt to spans, not structures
                if (source instanceof SSpan && target instanceof SToken) {
                    rel = docGraph.createRelation(source, target, SALT_TYPE.SSPANNING_RELATION, null);
                }
                break;
            }

            if (rel != null) {
                rel.setType(edgeType);

                // map edge labels
                Map<Pair<String, String>, String> labels = new LinkedHashMap<>();
                AnnisEdge.ByValue copyEdge = new AnnisEdge.ByValue();
                copyEdge.source = origEdge.source;
                copyEdge.target = origEdge.target;
                CAPI.AnnisVec_AnnisAnnotation annos = CAPI.annis_graph_edge_labels(orig, copyEdge, component);
                for (long i = 0; i < CAPI.annis_vec_annotation_size(annos).longValue(); i++) {
                    AnnisAnnotation.ByReference a = CAPI.annis_vec_annotation_get(annos, new NativeLong(i));

                    String ns = CAPI.annis_graph_str(orig, a.key.ns).toString();
                    String name = CAPI.annis_graph_str(orig, a.key.name).toString();
                    String value = CAPI.annis_graph_str(orig, a.value).toString();

                    if (name != null && value != null) {
                        if (ns == null) {
                            labels.put(new ImmutablePair<>("", name), value);
                        } else {
                            labels.put(new ImmutablePair<>(ns, name), value);
                        }
                    }
                }
                annos.dispose();
                mapLabels(rel, labels, false);

                AnnisString layerNameRaw = CAPI.annis_component_layer(component);
                String layerName = layerNameRaw == null ? null : layerNameRaw.toString();
                if (layerName != null && !layerName.isEmpty()) {
                    List<SLayer> layer = docGraph.getLayerByName(layerName);
                    if (layer == null || layer.isEmpty()) {
                        SLayer newLayer = SaltFactory.createSLayer();
                        newLayer.setName(layerName);
                        docGraph.addLayer(newLayer);
                        layer = Arrays.asList(newLayer);
                    }
                    layer.get(0).addRelation(rel);
                }
            }
        }
    }

    private void addNodeLayers() {
        List<SNode> nodeList = new LinkedList<>(docGraph.getNodes());
        for (SNode n : nodeList) {
            SFeature featLayer = n.getFeature("annis", "layer");
            if (featLayer != null) {
                SLayer layer = docGraph.getLayer(featLayer.getValue_STEXT());
                if (layer == null) {
                    layer = SaltFactory.createSLayer();
                    layer.setName(featLayer.getValue_STEXT());
                    docGraph.addLayer(layer);
                }
                layer.addNode(n);
            }
        }
    }

    private boolean mapTimeline(AnnisVec_AnnisComponent components) {

        // find all order relation types
        final int component_size = CAPI.annis_vec_component_size(components).intValue();

        Map<String, AnnisComponentConst> types = new TreeMap<>();
        for (int i = 0; i < component_size; i++) {
            AnnisComponentConst c = CAPI.annis_vec_component_get(components, new NativeLong(i));
            if (CAPI.annis_component_type(c) == AnnisComponentType.Ordering) {
                AnnisString t = CAPI.annis_component_name(c);
                types.put(t == null ? "" : t.toString(), c);
            }
        }

        AnnisComponentConst timelineOrderComponent = types.get("");
        if (types.size() > 1 && timelineOrderComponent != null) {

            // there can be only one outgoing edge at maximum per node, thus use a map
            // instead of a multimap
            Map<Integer, Integer> timelineOrderEdges = new HashMap<>();

            for (Map.Entry<Integer, SNode> node : nodesByID.entrySet()) {
                AnnisVec_AnnisEdge edges = CAPI.annis_graph_outgoing_edges(orig, new NodeID(node.getKey()),
                        timelineOrderComponent);
                final int edges_size = CAPI.annis_vec_edge_size(edges).intValue();
                for (int i = 0; i < edges_size; i++) {
                    AnnisEdge e = CAPI.annis_vec_edge_get(edges, new NativeLong(i));

                    timelineOrderEdges.put(e.source.intValue(), e.target.intValue());
                }
            }

            Set<Integer> rootNodes = new HashSet<>(timelineOrderEdges.keySet());
            for (Map.Entry<Integer, Integer> edge : timelineOrderEdges.entrySet()) {
                rootNodes.remove(edge.getValue());
            }

            if (rootNodes.size() == 1) {

                STimeline timeline = docGraph.createTimeline();

                // iterate over the edges of the component starting at the root node
                // (there should be only one chain of order relations for the timeline)
                Integer currentNode = rootNodes.iterator().next();
                while (currentNode != null) {

                    // Items on the timeline are not nodes, but "point of times", thus remove the
                    // node from
                    // map so it won't be added as SToken later.
                    SNode token = nodesByID.remove(currentNode);
                    if (token != null) {

                        // add another point of time for this token
                        timeline.increasePointOfTime();

                        // remember which token node (by ID) should be mapped to which timeline POT
                        node2timelinePOT.put(currentNode, timeline.getEnd());
                    }

                    // get next node in the chain
                    currentNode = timelineOrderEdges.get(currentNode);
                }

                return true;
            }
        }

        return false;
    }

    private void recreateText(final String name, List<SNode> rootNodes) {
        final StringBuilder text = new StringBuilder();
        final STextualDS ds = docGraph.createTextualDS("");

        ds.setName(name);

        Map<SToken, Range<Integer>> token2Range = new HashMap<>();

        // traverse the token chain using the order relations
        docGraph.traverse(rootNodes, SGraph.GRAPH_TRAVERSE_TYPE.TOP_DOWN_DEPTH_FIRST, "ORDERING_" + name,
                new GraphTraverseHandler() {
                    @Override
                    public void nodeReached(SGraph.GRAPH_TRAVERSE_TYPE traversalType, String traversalId,
                            SNode currNode, SRelation<SNode, SNode> relation, SNode fromNode, long order) {
                        if (fromNode != null) {
                            text.append(" ");
                        }

                        SFeature featTok = currNode.getFeature("annis::tok");
                        if (featTok != null && currNode instanceof SToken) {
                            int idxStart = text.length();
                            text.append(featTok.getValue_STEXT());
                            token2Range.put((SToken) currNode, Range.closed(idxStart, text.length()));
                        }
                    }

                    @Override
                    public void nodeLeft(SGraph.GRAPH_TRAVERSE_TYPE traversalType, String traversalId, SNode currNode,
                            SRelation<SNode, SNode> relation, SNode fromNode, long order) {
                    }

                    @Override
                    public boolean checkConstraint(SGraph.GRAPH_TRAVERSE_TYPE traversalType, String traversalId,
                            SRelation relation, SNode currNode, long order) {
                        if (relation == null) {
                            // TODO: check if this is ever true
                            return true;
                        } else if (relation instanceof SOrderRelation && Objects.equal(name, relation.getType())) {
                            return true;
                        } else {
                            return false;
                        }
                    }
                });

        // update the actual text
        ds.setText(text.toString());

        // add all relations to the text
        token2Range.forEach((t, r) -> {
            STextualRelation rel = SaltFactory.createSTextualRelation();
            rel.setSource(t);
            rel.setTarget(ds);
            rel.setStart(r.lowerEndpoint());
            rel.setEnd(r.upperEndpoint());
            docGraph.addRelation(rel);
        });

        if (docGraph.getTimeline() != null) {
            AnnisVec_AnnisComponent coverageComponents = CAPI.annis_graph_all_components_by_type(orig,
                    AnnisComponentType.Coverage);
            final int coverageComponents_size = CAPI.annis_vec_component_size(coverageComponents).intValue();
            AnnisComponentConst covComponent = null;
            for (int i = 0; i < coverageComponents_size; i++) {
                AnnisComponentConst c = CAPI.annis_vec_component_get(coverageComponents, new NativeLong(i));
                AnnisString cName = CAPI.annis_component_name(c);
                AnnisString cLayer = CAPI.annis_component_layer(c);
                if ((cName == null || cName.toString().isEmpty()) && "annis".equals(cLayer.toString())) {
                    covComponent = c;
                    break;
                }
            }

            if (covComponent != null) {
                // create the relations to the timeline for the tokens of this text by getting
                // the original node IDs of the coverage edges and their mapping to a point of time (POT)
                for (SToken tok : token2Range.keySet()) {
                    Integer tokID = nodesByID.inverse().get(tok);
                    if (tokID != null) {
                        AnnisVec_AnnisEdge edges = CAPI.annis_graph_outgoing_edges(orig, new NodeID(tokID),
                                covComponent);
                        final int edges_size = CAPI.annis_vec_edge_size(edges).intValue();
                        for(int i=0; i < edges_size; i++) {
                            AnnisEdge e = CAPI.annis_vec_edge_get(edges, new NativeLong(i));
                            
                            Integer pot = this.node2timelinePOT.get(e.target.intValue());
                            if(pot != null) {
                                STimelineRelation rel = SaltFactory.createSTimelineRelation();
                                rel.setSource(tok);
                                rel.setTarget(docGraph.getTimeline());
                                rel.setStart(pot);
                                rel.setEnd(pot);
                                docGraph.addRelation(rel);
                            }
                        }
                    }
                }
            }
        }
    }

    public static SDocumentGraph map(CAPI.AnnisGraphDB orig) {
        if (orig == null) {
            return null;
        }
        SaltExport export = new SaltExport(orig);

        export.mapDocGraph();
        return export.docGraph;
    }

    private void mapDocGraph() {
        // create all new nodes
        CAPI.AnnisIterPtr_AnnisNodeID itNodes = CAPI.annis_graph_nodes_by_type(orig, "node");

        if (itNodes != null) {

            for (NodeIDByRef nID = CAPI.annis_iter_nodeid_next(itNodes); nID != null; nID = CAPI
                    .annis_iter_nodeid_next(itNodes)) {
                SNode n = mapNode(nID);
                nodesByID.put(nID.getValue(), n);
            }
            itNodes.dispose();
        }

        AnnisVec_AnnisComponent components = CAPI.annis_graph_all_components(orig);
        final int component_size = CAPI.annis_vec_component_size(components).intValue();

        // check if timeline needs to be created
        mapTimeline(components);

        // add nodes to the graph
        nodesByID.values().stream().forEach(n -> docGraph.addNode(n));

        // create and add all edges
        for (Map.Entry<Integer, SNode> nodeEntry : nodesByID.entrySet()) {
            for (int i = 0; i < component_size; i++) {
                CAPI.AnnisComponentConst c = CAPI.annis_vec_component_get(components, new NativeLong(i));
                NodeID nId = new NodeID(nodeEntry.getKey());
                AnnisVec_AnnisEdge outEdges = CAPI.annis_graph_outgoing_edges(orig, nId, c);
                for (int edgeIdx = 0; edgeIdx < CAPI.annis_vec_edge_size(outEdges).intValue(); edgeIdx++) {
                    AnnisEdge edge = CAPI.annis_vec_edge_get(outEdges, new NativeLong(edgeIdx));
                    mapAndAddEdge(nId, edge, c);
                }
            }
        }

        // find all chains of SOrderRelations and reconstruct the texts belonging to
        // them
        Multimap<String, SNode> orderRoots = docGraph.getRootsByRelationType(SALT_TYPE.SORDER_RELATION);
        orderRoots.keySet().forEach((name) -> {
            ArrayList<SNode> roots = new ArrayList<>(orderRoots.get(name));
            if (SaltUtil.SALT_NULL_VALUE.equals(name)) {
                name = null;
            }
            if (name != null || docGraph.getTimeline() == null) {
                // only re-create text if there is no timeline or this is the non-default
                // tokenization
                recreateText(name, roots);
            }
        });

        addNodeLayers();
    }

    private static SCorpus addCorpusAndParents(SCorpusGraph cg, long id, Map<Long, Long> parentOfNode,
            Map<Long, SCorpus> id2corpus, Map<Long, Map<Pair<String, String>, String>> node2labels) {

        if (id2corpus.containsKey(id)) {
            return id2corpus.get(id);
        }

        Map<Pair<String, String>, String> labels = node2labels.get(id);
        if (labels == null) {
            return null;
        }

        // create parents first
        Long parentID = parentOfNode.get(id);
        SCorpus parent = null;
        if (parentID != null) {
            parent = addCorpusAndParents(cg, parentID, parentOfNode, id2corpus, node2labels);
        }

        String corpusName = labels.getOrDefault(new ImmutablePair<>("annis", "node_name"), "corpus");
        List<String> corpusNameSplitted = Splitter.on('/').trimResults().splitToList(corpusName);
        // use last part of the path as name
        SCorpus newCorpus = cg.createCorpus(parent, corpusNameSplitted.get(corpusNameSplitted.size() - 1));
        id2corpus.put(id, newCorpus);

        return newCorpus;

    }

    public static SCorpusGraph mapCorpusGraph(CAPI.AnnisGraphDB orig) {
        if (orig == null) {
            return null;
        }
        SCorpusGraph cg = SaltFactory.createSCorpusGraph();

        // find the part-of-subcorpus component
        AnnisVec_AnnisComponent components = CAPI.annis_graph_all_components(orig);
        AnnisComponentConst subcorpusComponent = null;
        for (int i = 0; i < CAPI.annis_vec_component_size(components).intValue(); i++) {
            CAPI.AnnisComponentConst c = CAPI.annis_vec_component_get(components, new NativeLong(i));
            if (CAPI.annis_component_type(c) == AnnisComponentType.PartOfSubcorpus) {
                subcorpusComponent = c;
            }
        }

        Map<Long, Map<Pair<String, String>, String>> node2labels = new LinkedHashMap<>();
        Map<Long, Long> parentOfNode = new LinkedHashMap<>();

        // iterate over all nodes and get their outgoing edges
        CAPI.AnnisIterPtr_AnnisNodeID itNodes = CAPI.annis_graph_nodes_by_type(orig, "corpus");
        if (itNodes != null) {

            for (NodeIDByRef nID = CAPI.annis_iter_nodeid_next(itNodes); nID != null; nID = CAPI
                    .annis_iter_nodeid_next(itNodes)) {

                Map<Pair<String, String>, String> nodeLabels = getNodeLabels(orig, nID.getValue());
                node2labels.put((long) nID.getValue(), nodeLabels);

                if (subcorpusComponent != null) {
                    AnnisVec_AnnisEdge outEdges = CAPI.annis_graph_outgoing_edges(orig, new NodeID(nID.getValue()),
                            subcorpusComponent);
                    for (int edgeIdx = 0; edgeIdx < CAPI.annis_vec_edge_size(outEdges).intValue(); edgeIdx++) {
                        AnnisEdge edge = CAPI.annis_vec_edge_get(outEdges, new NativeLong(edgeIdx));
                        // add edge
                        parentOfNode.put(edge.source.longValue(), edge.target.longValue());
                    }
                }
            }
            itNodes.dispose();
        }

        if (parentOfNode.isEmpty()) {
            // if there are no edges at all, there are only root corpora (or a single root
            // corpus)
            for (Map<Pair<String, String>, String> labels : node2labels.values()) {

                String corpusName = labels.getOrDefault(new ImmutablePair<>("annis", "node_name"), "corpus");
                List<String> corpusNameSplitted = Splitter.on('/').trimResults().splitToList(corpusName);
                // use last part of the path as name
                SCorpus rootCorpus = cg.createCorpus(null, corpusNameSplitted.get(corpusNameSplitted.size() - 1));
                mapLabels(rootCorpus, labels, true);
            }

        } else {

            Map<Long, SCorpus> id2corpus = new HashMap<>();
            // add all non-documents first
            for (Long id : parentOfNode.values()) {
                addCorpusAndParents(cg, id, parentOfNode, id2corpus, node2labels);
            }
            for (Map.Entry<Long, SCorpus> e : id2corpus.entrySet()) {
                Map<Pair<String, String>, String> labels = node2labels.get(e.getKey());
                if (labels != null) {
                    mapLabels(e.getValue(), labels, true);
                }
            }

            // add all documents next
            for (Map.Entry<Long, Long> edge : parentOfNode.entrySet()) {
                long childID = edge.getKey();
                long parentID = edge.getValue();
                if (!id2corpus.containsKey(childID)) {
                    Map<Pair<String, String>, String> labels = node2labels.get(childID);
                    if (labels != null) {
                        String docName = labels.getOrDefault(new ImmutablePair<>("annis", "doc"), "document");

                        SCorpus parent = id2corpus.get(parentID);
                        SDocument doc = cg.createDocument(parent, docName);

                        mapLabels(doc, labels, true);
                    }
                }
            }
        }

        return cg;
    }
}
