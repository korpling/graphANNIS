/*
 * Copyright 2018 Thomas Krause.
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
package org.corpus_tools.graphannis.api;

import org.corpus_tools.graphannis.capi.CAPI;

/**
 * An API for applying atomic updates to a graph DB.
 *    
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class GraphUpdate {
    private final CAPI.AnnisGraphUpdate instance;

    public GraphUpdate() {
        this.instance = CAPI.annis_graphupdate_new();
    }

    public CAPI.AnnisGraphUpdate getInstance() {
        return this.instance;
    }

    public void addNode(String node_name, String node_type) {
        CAPI.annis_graphupdate_add_node(instance, node_name, node_type);
    }

    public void addNode(String node_name) {
        CAPI.annis_graphupdate_add_node(instance, node_name, "node");
    }

    public void deleteNode(String node_name) {
        CAPI.annis_graphupdate_delete_node(instance, node_name);
    }

    public void addNodeLabel(String node_name, String anno_ns, String anno_name, String anno_value) {
        CAPI.annis_graphupdate_add_node_label(instance, node_name, anno_ns, anno_name, anno_value);
    }

    public void deleteNodeLabel(String node_name, String anno_ns, String anno_name) {
        CAPI.annis_graphupdate_delete_node_label(instance, node_name, anno_ns, anno_name);
    }

    public void addEdge(String source_node, String target_node, String layer, String component_type,
            String component_name) {
        CAPI.annis_graphupdate_add_edge(instance, source_node, target_node, layer, component_type, component_name);
    }

    public void deleteEdge(String source_node, String target_node, String layer, String component_type,
            String component_name) {
        CAPI.annis_graphupdate_add_edge(instance, source_node, target_node, layer, component_type, component_name);
    }

    public void addEdgeLabel(String source_node, String target_node, String layer, String component_type,
            String component_name, String anno_ns, String anno_name, String anno_value) {
        CAPI.annis_graphupdate_add_edge_label(instance, source_node, target_node, layer, component_type, component_name,
                anno_ns, anno_name, anno_value);
    }

    public void deleteEdgeLabel(String source_node, String target_node, String layer, String component_type,
            String component_name, String anno_ns, String anno_name) {
        CAPI.annis_graphupdate_delete_edge_label(instance, source_node, target_node, layer, component_type,
                component_name, anno_ns, anno_name);
    }
}