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
import com.sun.jna.Pointer;
import com.sun.jna.PointerType;

public class CAPI implements Library
{

    public static class AnnisPtr extends PointerType
    {

    }

    public static class AnnisCorpusStorage extends AnnisPtr
    {
    }

    public static class AnnisGraphUpdate extends AnnisPtr
    {
    }

    public static class AnnisVec_AnnisCString extends AnnisPtr
    {
    }

    public static class AnnisError extends AnnisPtr
    {
    }

    public static native void annis_free(AnnisPtr ptr);

    public static native void annis_str_free(AnnisPtr ptr);

    public static native AnnisCorpusStorage annis_cs_new(String db_dir);

    public static native AnnisVec_AnnisCString annis_cs_list(AnnisCorpusStorage cs);

    public static native long annis_cs_count(AnnisCorpusStorage cs, String corpusName,
            String queryAsJSON);

    public static native AnnisVec_AnnisCString annis_cs_find(AnnisCorpusStorage cs,
            String corpusName, String queryAsJSON, long offset, long limit);

    public static native AnnisError annis_cs_apply_update(AnnisCorpusStorage cs, String corpusName,
            AnnisGraphUpdate update);

    public static native AnnisGraphUpdate annis_graphupdate_new();

    public static native void annis_graphupdate_add_node(AnnisGraphUpdate ptr, String node_name,
            String node_type);

    public static native void annis_graphupdate_delete_node(AnnisGraphUpdate ptr, String node_name);

    public static native void annis_graphupdate_add_node_label(AnnisGraphUpdate ptr,
            String node_name, String anno_ns, String anno_name, String anno_value);

    public static native void annis_graphupdate_delete_node_label(AnnisGraphUpdate ptr,
            String node_name, String anno_ns, String anno_name);

    public static native void annis_graphupdate_add_edge(AnnisGraphUpdate ptr, String source_node,
            String target_node, String layer, String component_type, String component_name);

    public static native void annis_graphupdate_delete_edge(AnnisGraphUpdate ptr,
            String source_node, String target_node, String layer, String component_type,
            String component_name);

    public static native void annis_graphupdate_add_edge_label(AnnisGraphUpdate ptr,
            String source_node, String target_node, String layer, String component_type,
            String component_name, String anno_ns, String anno_name, String anno_value);

    public static native void annis_graphupdate_delete_edge_label(AnnisGraphUpdate ptr,
            String source_node, String target_node, String layer, String component_type,
            String component_name, String anno_ns, String anno_name);

    public static native String annis_error_get_msg(AnnisError ptr);

    public static native long annis_vec_str_size(AnnisVec_AnnisCString ptr);

    public static native String annis_vec_str_get(AnnisVec_AnnisCString ptr, long i);

    static
    {
        Native.register(CAPI.class, "graphannis_capi");
    }

}
