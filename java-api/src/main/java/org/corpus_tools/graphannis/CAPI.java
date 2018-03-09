/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

package org.corpus_tools.graphannis;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.PointerType;

public interface CAPI extends Library
{

    CAPI INSTANCE = (CAPI) Native.loadLibrary("graphannis_capi", CAPI.class);

    public static class ANNIS_CorpusStorage extends PointerType
    {
    }

    public static class ANNIS_GraphUpdate extends PointerType
    {
    }

    public ANNIS_CorpusStorage annis_cs_new(String db_dir);

    public void annis_cs_free(ANNIS_CorpusStorage cs);

    public long annis_cs_count(ANNIS_CorpusStorage cs, String corpusName, String queryAsJSON);
    public void annis_cs_apply_update(ANNIS_CorpusStorage cs, String corpusName, ANNIS_GraphUpdate update);

    public ANNIS_GraphUpdate annis_graphupdate_new();

    public void annis_graphupdate_free(ANNIS_GraphUpdate ptr);

    public void annis_graphupdate_add_node(ANNIS_GraphUpdate ptr, String node_name,
            String node_type);

    public void annis_graphupdate_delete_node(ANNIS_GraphUpdate ptr, String node_name);

    public void annis_graphupdate_add_node_label(ANNIS_GraphUpdate ptr, String node_name,
            String anno_ns, String anno_name, String anno_value);

    public void annis_graphupdate_delete_node_label(ANNIS_GraphUpdate ptr, String node_name,
            String anno_ns, String anno_name);

    public void annis_graphupdate_add_edge(ANNIS_GraphUpdate ptr, String source_node,
            String target_node, String layer, String component_type, String component_name);

    public void annis_graphupdate_delete_edge(ANNIS_GraphUpdate ptr, String source_node,
            String target_node, String layer, String component_type, String component_name);

    public void annis_graphupdate_add_edge_label(ANNIS_GraphUpdate ptr, String source_node,
            String target_node, String layer, String component_type, String component_name,
            String anno_ns, String anno_name, String anno_value);

    public void annis_graphupdate_delete_edge_label(ANNIS_GraphUpdate ptr, String source_node,
            String target_node, String layer, String component_type, String component_name,
            String anno_ns, String anno_name);

}
