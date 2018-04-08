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

import com.sun.jna.NativeLong;
import java.util.ArrayList;
import java.util.List;
import org.corpus_tools.graphannis.SaltExport;
import org.corpus_tools.graphannis.capi.AnnisCountExtra;
import org.corpus_tools.graphannis.capi.CAPI;
import org.corpus_tools.salt.common.SCorpusGraph;
import org.corpus_tools.salt.common.SDocumentGraph;

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
        if(instance != null) {
            CAPI.AnnisGraphDB graph = CAPI.annis_cs_corpus_graph(instance, corpusName);
            
            SCorpusGraph result = SaltExport.mapCorpusGraph(graph);
            if(graph != null) {
                graph.dispose();
            }
            return result;
        }
        return null;
    }
    

    public void importRelANNIS(String corpusName, String path) {
        CAPI.AnnisError result = CAPI.annis_cs_import_relannis(instance, corpusName, path);
        if (result != null) {
            String msg = CAPI.annis_error_get_msg(result);
            result.dispose();

            throw new RuntimeException(msg);
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
