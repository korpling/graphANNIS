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

  public static class AnnisCorpusStorage extends PointerType
  {
  }

  public static class AnnisGraphUpdate extends PointerType
  {
  }
  
  public static class AnnisVec_AnnisCString extends PointerType
  {
  }
  
  public static class AnnisError extends PointerType
  {
  }

  public void annis_free(PointerType ptr);
  
  public void annis_str_free(PointerType ptr);

  public AnnisCorpusStorage annis_cs_new(String db_dir);


  public AnnisVec_AnnisCString annis_cs_list(AnnisCorpusStorage cs);

  public long annis_cs_count(AnnisCorpusStorage cs, String corpusName, String queryAsJSON);
  public AnnisVec_AnnisCString annis_cs_find(AnnisCorpusStorage cs, String corpusName, String queryAsJSON, long offset, long limit);

  public AnnisError annis_cs_apply_update(AnnisCorpusStorage cs, String corpusName,
      AnnisGraphUpdate update);

  public AnnisGraphUpdate annis_graphupdate_new();

  public void annis_graphupdate_add_node(AnnisGraphUpdate ptr, String node_name, String node_type);

  public void annis_graphupdate_delete_node(AnnisGraphUpdate ptr, String node_name);

  public void annis_graphupdate_add_node_label(AnnisGraphUpdate ptr, String node_name,
      String anno_ns, String anno_name, String anno_value);

  public void annis_graphupdate_delete_node_label(AnnisGraphUpdate ptr, String node_name,
      String anno_ns, String anno_name);

  public void annis_graphupdate_add_edge(AnnisGraphUpdate ptr, String source_node,
      String target_node, String layer, String component_type, String component_name);

  public void annis_graphupdate_delete_edge(AnnisGraphUpdate ptr, String source_node,
      String target_node, String layer, String component_type, String component_name);

  public void annis_graphupdate_add_edge_label(AnnisGraphUpdate ptr, String source_node,
      String target_node, String layer, String component_type, String component_name,
      String anno_ns, String anno_name, String anno_value);

  public void annis_graphupdate_delete_edge_label(AnnisGraphUpdate ptr, String source_node,
      String target_node, String layer, String component_type, String component_name,
      String anno_ns, String anno_name);
  

  public String annis_error_get_msg(AnnisError ptr);
  
  public long annis_stringvec_size(AnnisVec_AnnisCString ptr);
  public String annis_stringvec_get(AnnisVec_AnnisCString ptr, long i);
  

}
