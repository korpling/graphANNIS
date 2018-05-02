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

import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;

import org.corpus_tools.graphannis.QueryToJSON;
import org.corpus_tools.graphannis.SaltExport;
import org.corpus_tools.graphannis.capi.AnnisComponentType;
import org.corpus_tools.graphannis.capi.AnnisCountExtra;
import org.corpus_tools.graphannis.capi.AnnisString;
import org.corpus_tools.graphannis.capi.CAPI;
import org.corpus_tools.graphannis.capi.CAPI.AnnisComponentConst;
import org.corpus_tools.salt.common.SCorpusGraph;
import org.corpus_tools.salt.common.SDocumentGraph;

import com.sun.jna.NativeLong;

import annis.model.Annotation;

/**
 * An API for managing corpora stored in a common location on the file system.
 * 
 * @author Thomas Krause <thomaskrause@posteo.de>
 */
public class CorpusStorageManager {
    private final CAPI.AnnisCorpusStorage instance;

    public static class CountResult {
        public long matchCount;
        public long documentCount;
    }

    public CorpusStorageManager(String dbDir) {
        this(dbDir, null, LogLevel.Off);
    }

    public CorpusStorageManager(String dbDir, String logfile, LogLevel level) {
        CAPI.annis_init_logging(logfile, level.getRaw());
        this.instance = CAPI.annis_cs_new(dbDir);
    }

    public String[] list() {
        CAPI.AnnisVec_AnnisCString orig = CAPI.annis_cs_list(instance);
        String[] copy = new String[CAPI.annis_vec_str_size(orig).intValue()];
        for (int i = 0; i < copy.length; i++) {
            copy[i] = CAPI.annis_vec_str_get(orig, new NativeLong(i));
        }

        orig.dispose();

        return copy;
    }

    public List<Annotation> listNodeAnnotations(String corpusName, boolean listValues, boolean onlyMostFrequentValues) {
        List<Annotation> result = new LinkedList<>();
        if (instance != null) {
            CAPI.AnnisMatrix_AnnisCString orig = CAPI.annis_cs_list_node_annotations(instance, corpusName, listValues,
                    onlyMostFrequentValues);

            final int nrows = CAPI.annis_matrix_str_nrows(orig).intValue();
            final int ncols = CAPI.annis_matrix_str_ncols(orig).intValue();
            if (ncols >= (listValues ? 3 : 2)) {
                for (int i = 0; i < nrows; i++) {
                    Annotation anno = new Annotation();
                    String ns = CAPI.annis_matrix_str_get(orig, new NativeLong(i), new NativeLong(0));
                    String name = CAPI.annis_matrix_str_get(orig, new NativeLong(i), new NativeLong(1));
                    
                    if(!"".equals(ns)) {
                        anno.setNamespace(ns);
                    }
                    anno.setName(name);
                    if(listValues) {
                        String val = CAPI.annis_matrix_str_get(orig, new NativeLong(i), new NativeLong(2));
                        anno.setValue(val);
                    }
                    result.add(anno);
                }
            }

            orig.dispose();
        }
        return result;
    }

    public List<String> getAllOrderRelationNames(String corpusName) {
        List<String> result = new LinkedList<>();
        if (instance != null) {
            CAPI.AnnisVec_AnnisComponent orig = CAPI.annis_cs_all_components_by_type(instance, corpusName,
                    AnnisComponentType.Ordering);

            for (int i = 0; i < CAPI.annis_vec_component_size(orig).intValue(); i++) {
                AnnisComponentConst c = CAPI.annis_vec_component_get(orig, new NativeLong(i));
                AnnisString cname = CAPI.annis_component_name(c);
                if (cname != null) {
                    result.add(cname.toString());
                }
            }
        }
        return result;
    }

    public long count(List<String> corpora, String queryAsJSON) {
        long result = 0l;
        for (String corpusName : corpora) {
            result += CAPI.annis_cs_count(instance, corpusName, queryAsJSON);
        }
        return result;
    }

    public CountResult countExtra(List<String> corpora, String queryAsJSON) {
        CountResult result = new CountResult();
        result.documentCount = 0;
        result.matchCount = 0;
        for (String corpusName : corpora) {
            AnnisCountExtra resultForCorpus = CAPI.annis_cs_count_extra(instance, corpusName, queryAsJSON);
            result.matchCount += resultForCorpus.matchCount;
            result.documentCount += resultForCorpus.documentCount;
        }
        return result;
    }

    public String[] find(List<String> corpora, String queryAsJSON, long offset, long limit) {
        ArrayList<String> result = new ArrayList<>();
        for (String corpusName : corpora) {
            CAPI.AnnisVec_AnnisCString vec = CAPI.annis_cs_find(instance, corpusName, queryAsJSON, offset, limit);
            final int vecSize = CAPI.annis_vec_str_size(vec).intValue();
            for (int i = 0; i < vecSize; i++) {
                result.add(CAPI.annis_vec_str_get(vec, new NativeLong(i)));
            }
            vec.dispose();
        }

        return result.toArray(new String[0]);
    }

    public SDocumentGraph subgraph(String corpusName, List<String> node_ids, long ctx_left, long ctx_right) {
        CAPI.AnnisVec_AnnisCString c_node_ids = CAPI.annis_vec_str_new();
        for (String id : node_ids) {
            CAPI.annis_vec_str_push(c_node_ids, id);
        }
        CAPI.AnnisGraphDB graph = CAPI.annis_cs_subgraph(instance, corpusName, c_node_ids, new NativeLong(ctx_left),
                new NativeLong(ctx_right));

        SDocumentGraph result = SaltExport.map(graph);
        c_node_ids.dispose();
        graph.dispose();

        return result;
    }

    public SDocumentGraph subcorpusGraph(String corpusName, List<String> document_ids) {
        CAPI.AnnisVec_AnnisCString c_document_ids = CAPI.annis_vec_str_new();
        for (String id : document_ids) {
            CAPI.annis_vec_str_push(c_document_ids, id);
        }

        SDocumentGraph result = null;
        if (instance != null) {
            CAPI.AnnisGraphDB graph = CAPI.annis_cs_subcorpus_graph(instance, corpusName, c_document_ids);

            result = SaltExport.map(graph);
            c_document_ids.dispose();
            if (graph != null) {
                graph.dispose();
            }
        }

        return result;
    }

    public SCorpusGraph corpusGraph(String corpusName) {
        if (instance != null) {
            CAPI.AnnisGraphDB graph = CAPI.annis_cs_corpus_graph(instance, corpusName);

            SCorpusGraph result = SaltExport.mapCorpusGraph(graph);
            if (graph != null) {
                graph.dispose();
            }
            return result;
        }
        return null;
    }

    public SCorpusGraph corpusGraphForQuery(String corpusName, String aql) {
        if (instance != null) {
            String json = QueryToJSON.aqlToJSON(aql);
            CAPI.AnnisGraphDB graph = CAPI.annis_cs_subgraph_for_query(instance, corpusName, json);

            SCorpusGraph result = SaltExport.mapCorpusGraph(graph);
            if (graph != null) {
                graph.dispose();
            }
            return result;
        }
        return null;
    }

    public SDocumentGraph subGraphForQuery(String corpusName, String aql) {
        if (instance != null) {
            CAPI.AnnisGraphDB graph = CAPI.annis_cs_subgraph_for_query(instance, corpusName,
                    QueryToJSON.aqlToJSON(aql));

            SDocumentGraph result = SaltExport.map(graph);
            if (graph != null) {
                graph.dispose();
            }
            return result;
        }
        return null;
    }

    public void importRelANNIS(String corpusName, String path) {
        if (instance != null) {
            CAPI.AnnisError result = CAPI.annis_cs_import_relannis(instance, corpusName, path);
            if (result != null) {
                String msg = CAPI.annis_error_get_msg(result);
                result.dispose();

                throw new RuntimeException(msg);
            }
        }
    }

    public void deleteCorpus(String corpusName) {
        if (instance != null) {
            CAPI.annis_cs_delete(instance, corpusName);
        }
    }

    public void applyUpdate(String corpusName, GraphUpdate update) {
        CAPI.AnnisError result = CAPI.annis_cs_apply_update(instance, corpusName, update.getInstance());

        if (result != null) {
            String msg = CAPI.annis_error_get_msg(result);
            result.dispose();

            throw new RuntimeException(msg);
        }
    }

}
