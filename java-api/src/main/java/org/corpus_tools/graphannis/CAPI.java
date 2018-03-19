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

import java.util.Arrays;
import java.util.List;

import javax.xml.soap.Node;

import com.sun.jna.IntegerType;
import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.NativeLong;
import com.sun.jna.PointerType;
import com.sun.jna.Structure;
import com.sun.jna.Structure.ByReference;

import annis.exceptions.AnnisQLSemanticsException;



public class CAPI implements Library
{

  static
  {
    Native.register(CAPI.class, "graphannis_capi");
  }

  public static class NodeID extends IntegerType {
    public static final int SIZE = 4;
    public NodeID()
    {
      this(0);
    }
    public NodeID(int value)
    {
      super(SIZE, value, false);
    }

    public static class ByRef extends NodeID implements ByReference {
			
		};
  }
  public static class StringID extends IntegerType {
    public static final int SIZE = 4;
    public StringID()
    {
      this(0);
    }
    public StringID(int value)
    {
      super(SIZE, value, false);
    }
    public static class ByRef extends StringID implements ByReference {
			
		};
  }

  public static class AnnisPtr extends PointerType
  {

  }

  public static class AnnisCorpusStorage extends AnnisPtr
  {
  }

  public static class AnnisGraphUpdate extends AnnisPtr
  {
  }

  public static class AnnisGraphDB extends AnnisPtr
  {
  }

  public static class AnnisVec_AnnisCString extends AnnisPtr
  {
  }

  public static class AnnisVec_AnnisAnnotation extends AnnisPtr
  {
  }

  public static class AnnisError extends AnnisPtr
  {
  }

  public static class AnnisIterPtr_AnnisNodeID extends AnnisPtr
  {

  }

  public static class AnnisAnnoKey extends Structure
  {
    public StringID name;
    public StringID ns;

    @Override
    protected List<String> getFieldOrder()
    {
      return Arrays.asList("name", "ns");
    }
    public static class ByReference extends AnnisAnnoKey implements Structure.ByReference {
			
		};
		public static class ByValue extends AnnisAnnoKey implements Structure.ByValue {
			
		};
  }

  public static class AnnisAnnotation extends Structure
  {

    public AnnisAnnoKey key;
    public StringID value;

    @Override
    protected List<String> getFieldOrder()
    {
      return Arrays.asList("key", "value");
    }
    public static class ByReference extends AnnisAnnoKey implements Structure.ByReference {
			
		};
		public static class ByValue extends AnnisAnnoKey implements Structure.ByValue {
			
		};
  }
  // general functions

  public static native void annis_free(AnnisPtr ptr);

  public static native void annis_str_free(AnnisPtr ptr);

  public static native String annis_error_get_msg(AnnisError ptr);

  // vector and iterator functions 
  public static native long annis_vec_str_size(AnnisVec_AnnisCString ptr);

  public static native String annis_vec_str_get(AnnisVec_AnnisCString ptr, NativeLong i);

  public static native long annis_vec_annotation_size(AnnisVec_AnnisAnnotation ptr);

  public static native AnnisAnnotation annis_vec_annotation_get(AnnisVec_AnnisAnnotation ptr,
      NativeLong i);

  public static native NodeID.ByRef annis_iter_nodeid_next(AnnisIterPtr_AnnisNodeID ptr); 

  // corpus storage class

  public static native AnnisCorpusStorage annis_cs_new(String db_dir);

  public static native AnnisVec_AnnisCString annis_cs_list(AnnisCorpusStorage cs);

  public static native long annis_cs_count(AnnisCorpusStorage cs, String corpusName,
      String queryAsJSON);

  public static native AnnisVec_AnnisCString annis_cs_find(AnnisCorpusStorage cs, String corpusName,
      String queryAsJSON, long offset, long limit);

  public static native AnnisError annis_cs_apply_update(AnnisCorpusStorage cs, String corpusName,
      AnnisGraphUpdate update);

  // graph update class

  public static native AnnisGraphUpdate annis_graphupdate_new();

  public static native void annis_graphupdate_add_node(AnnisGraphUpdate ptr, String node_name,
      String node_type);

  public static native void annis_graphupdate_delete_node(AnnisGraphUpdate ptr, String node_name);

  public static native void annis_graphupdate_add_node_label(AnnisGraphUpdate ptr, String node_name,
      String anno_ns, String anno_name, String anno_value);

  public static native void annis_graphupdate_delete_node_label(AnnisGraphUpdate ptr,
      String node_name, String anno_ns, String anno_name);

  public static native void annis_graphupdate_add_edge(AnnisGraphUpdate ptr, String source_node,
      String target_node, String layer, String component_type, String component_name);

  public static native void annis_graphupdate_delete_edge(AnnisGraphUpdate ptr, String source_node,
      String target_node, String layer, String component_type, String component_name);

  public static native void annis_graphupdate_add_edge_label(AnnisGraphUpdate ptr,
      String source_node, String target_node, String layer, String component_type,
      String component_name, String anno_ns, String anno_name, String anno_value);

  public static native void annis_graphupdate_delete_edge_label(AnnisGraphUpdate ptr,
      String source_node, String target_node, String layer, String component_type,
      String component_name, String anno_ns, String anno_name);

  // GraphDB classes

  public static native AnnisVec_AnnisAnnotation annis_graph_node_labels(AnnisGraphDB g, NodeID nodeID);
  public static native AnnisIterPtr_AnnisNodeID annis_graph_nodes_by_type(AnnisGraphDB g, String node_type);
  public static native String annis_graph_str(AnnisGraphDB g, StringID str_id);
}
